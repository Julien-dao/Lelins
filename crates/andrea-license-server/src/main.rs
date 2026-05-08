//! `andrea-license` — CLI to generate, verify, and inspect ANDREA license keys.
//!
//! Used by Julien (and later, the Gumroad webhook) to mint keys. The server
//! secret is read from the `ANDREA_LICENSE_SECRET` environment variable —
//! never pass it on the command line.
//!
//! # Examples
//!
//! ```text
//! ANDREA_LICENSE_SECRET=$(cat secret.bin) \
//!     andrea-license generate \
//!         --tier pro \
//!         --email buyer@example.com
//!
//! ANDREA_LICENSE_SECRET=$(cat secret.bin) \
//!     andrea-license verify \
//!         --key ANDREA-PRO-XXXXX-XXXXX-XXXXX-XXXXX \
//!         --email buyer@example.com
//! ```

use std::process::ExitCode;

use andrea_license::{
    email_hash, generate, verify, License, LicensePayload, Tier, CURRENT_VERSION,
};
use chrono::{NaiveDate, Utc};
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(
    name = "andrea-license",
    about = "Generate and verify ANDREA license keys.",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Generate a new license key.
    Generate(GenerateArgs),
    /// Verify a license key against a buyer's email.
    Verify(VerifyArgs),
    /// Decode a key without verifying its MAC (debug helper).
    Inspect(InspectArgs),
}

#[derive(Parser, Debug)]
struct GenerateArgs {
    /// Tier to encode in the key.
    #[arg(long)]
    tier: TierArg,
    /// Buyer email; the key will only validate for this exact email.
    #[arg(long)]
    email: String,
    /// Issuance date in YYYY-MM-DD (defaults to today, UTC).
    #[arg(long)]
    issued: Option<String>,
    /// Feature bitmask (16 bits, decimal or 0xHEX).
    #[arg(long, default_value = "0")]
    features: String,
    /// Output as JSON (default: plain key on stdout).
    #[arg(long)]
    json: bool,
}

#[derive(Parser, Debug)]
struct VerifyArgs {
    /// Key to verify.
    #[arg(long)]
    key: String,
    /// Email under which the key was issued.
    #[arg(long)]
    email: String,
    /// Output as JSON.
    #[arg(long)]
    json: bool,
}

#[derive(Parser, Debug)]
struct InspectArgs {
    /// Key to decode.
    #[arg(long)]
    key: String,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum TierArg {
    Discovery,
    Pro,
    Master,
    Bundle,
}

impl From<TierArg> for Tier {
    fn from(t: TierArg) -> Self {
        match t {
            TierArg::Discovery => Tier::Discovery,
            TierArg::Pro => Tier::Pro,
            TierArg::Master => Tier::Master,
            TierArg::Bundle => Tier::Bundle,
        }
    }
}

const REFERENCE_DATE: &str = "2026-01-01";

fn read_secret() -> Result<Vec<u8>, String> {
    std::env::var("ANDREA_LICENSE_SECRET")
        .map(|s| s.into_bytes())
        .map_err(|_| {
            "missing ANDREA_LICENSE_SECRET environment variable (32+ bytes recommended)".to_string()
        })
}

fn parse_features(raw: &str) -> Result<u16, String> {
    let trimmed = raw.trim();
    let parsed = if let Some(rest) = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
    {
        u32::from_str_radix(rest, 16)
    } else {
        trimmed.parse::<u32>()
    }
    .map_err(|e| format!("invalid features value: {e}"))?;
    if parsed > u16::MAX as u32 {
        return Err(format!("features value {parsed} exceeds 16 bits"));
    }
    Ok(parsed as u16)
}

fn issued_days_from(date_str: Option<&str>) -> Result<u16, String> {
    let reference: NaiveDate = REFERENCE_DATE
        .parse()
        .expect("hardcoded reference date is valid");
    let date: NaiveDate = match date_str {
        None => Utc::now().date_naive(),
        Some(s) => s
            .parse()
            .map_err(|e| format!("invalid issued date `{s}`: {e}"))?,
    };
    let delta = date.signed_duration_since(reference).num_days();
    if delta < 0 {
        return Err(format!(
            "issued date `{date}` is before reference {REFERENCE_DATE}"
        ));
    }
    if delta > u16::MAX as i64 {
        return Err(format!(
            "issued date is too far in the future ({delta} days, max {})",
            u16::MAX
        ));
    }
    Ok(delta as u16)
}

fn cmd_generate(args: GenerateArgs) -> Result<(), String> {
    let secret = read_secret()?;
    let features = parse_features(&args.features)?;
    let issued_days = issued_days_from(args.issued.as_deref())?;
    let payload = LicensePayload {
        version: CURRENT_VERSION,
        tier: args.tier.into(),
        email_hash: email_hash(&args.email),
        issued_days,
        features,
    };
    let key = generate(payload, &secret).map_err(|e| e.to_string())?;
    if args.json {
        let value = serde_json::json!({
            "key": key,
            "tier": payload.tier.label(),
            "email": args.email,
            "issued_days": issued_days,
            "features": features,
            "version": payload.version,
        });
        println!("{}", serde_json::to_string_pretty(&value).unwrap());
    } else {
        println!("{key}");
    }
    Ok(())
}

fn cmd_verify(args: VerifyArgs) -> Result<(), String> {
    let secret = read_secret()?;
    let license = verify(&args.key, &args.email, &secret, &[]).map_err(|e| e.to_string())?;
    let License { payload, canonical } = license;
    if args.json {
        let value = serde_json::json!({
            "valid": true,
            "canonical": canonical,
            "tier": payload.tier.label(),
            "version": payload.version,
            "issued_days": payload.issued_days,
            "features": payload.features,
        });
        println!("{}", serde_json::to_string_pretty(&value).unwrap());
    } else {
        println!(
            "OK — tier {}, issued day {}",
            payload.tier.label(),
            payload.issued_days
        );
    }
    Ok(())
}

fn cmd_inspect(args: InspectArgs) -> Result<(), String> {
    use andrea_license::{decode_payload, normalize, ENCODED_CHARS, MAC_BITS};
    let normalized = normalize(&args.key);
    if !normalized.starts_with("ANDREA") {
        return Err("key does not start with ANDREA".to_string());
    }
    let after_brand = &normalized[6..];
    // Find the tier label (3 or 4 chars).
    let mut tier_label = String::new();
    let mut payload_str = String::new();
    for tier_len in [4usize, 3] {
        if after_brand.len() >= tier_len + ENCODED_CHARS {
            let label = &after_brand[..tier_len];
            if Tier::from_label(label).is_ok() {
                tier_label = label.to_string();
                payload_str = after_brand[tier_len..tier_len + ENCODED_CHARS].to_string();
                break;
            }
        }
    }
    if tier_label.is_empty() {
        return Err("no recognized tier prefix".to_string());
    }
    let bits = decode_payload(&payload_str).map_err(|e| e.to_string())?;
    let mac_mask: u128 = (1u128 << MAC_BITS) - 1;
    let packed72 = bits & !mac_mask;
    let mac = (bits & mac_mask) as u32;
    let payload = LicensePayload::unpack72(packed72).map_err(|e| e.to_string())?;
    let value = serde_json::json!({
        "tier_label": tier_label,
        "version": payload.version,
        "tier": payload.tier.label(),
        "email_hash_hex": format!("{:08X}", payload.email_hash),
        "issued_days": payload.issued_days,
        "features": format!("0x{:04X}", payload.features),
        "mac_hex": format!("{:07X}", mac),
        "warning": "MAC NOT VERIFIED — use `verify` for that"
    });
    println!("{}", serde_json::to_string_pretty(&value).unwrap());
    Ok(())
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Generate(args) => cmd_generate(args),
        Command::Verify(args) => cmd_verify(args),
        Command::Inspect(args) => cmd_inspect(args),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("error: {msg}");
            ExitCode::FAILURE
        }
    }
}
