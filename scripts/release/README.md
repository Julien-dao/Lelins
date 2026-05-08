# Release scripts

Helpers consumed by `.github/workflows/release.yml`.

## `generate-latest-json.sh`

Builds the `latest.json` manifest read by `tauri-plugin-updater`. Expects
the per-target build artifacts to be downloaded under
`artifacts/andrea-<target-triple>/`. Reads the Minisign signature files
produced by Tauri's signing step (`*.sig`) verbatim.

```
NOTES="Bug fixes and faster onboarding" \
  ./generate-latest-json.sh v0.2.0 ./artifacts > latest.json
```

## First-time secrets setup (one-off, on Julien's machine)

1. **Apple Developer ID Application certificate** :
   - Generate the certificate in App Store Connect.
   - Export the `.p12` from Keychain Access including the private key.
   - `base64 -i certificate.p12 | pbcopy` and paste into the
     `APPLE_CERTIFICATE` repo secret.
   - Set `APPLE_SIGNING_IDENTITY` to the full identity string, e.g.
     `Developer ID Application: Julien PERROT (XXXXXXXXXX)`.
   - Generate an app-specific password in [appleid.apple.com](https://appleid.apple.com)
     for `APPLE_PASSWORD` (notarization).
   - Set `APPLE_TEAM_ID` to the 10-character team identifier.

2. **Windows Authenticode certificate** :
   - Buy / renew a code signing certificate (e.g. Sectigo).
   - Export as `.pfx`. Base64-encode it into `WINDOWS_CERTIFICATE`.
   - Set `WINDOWS_CERTIFICATE_PASSWORD`.

3. **Minisign key for the updater** :
   ```bash
   tauri signer generate -w minisign-private-key.txt
   # → prints the public key — paste it into apps/desktop/src-tauri/tauri.conf.json
   #   under plugins.updater.pubkey
   # → store the contents of minisign-private-key.txt as
   #   TAURI_SIGNING_PRIVATE_KEY (and the passphrase as
   #   TAURI_SIGNING_PRIVATE_KEY_PASSWORD)
   ```

4. **Server secret for the licence MAC** : long random bytes (≥ 32 bytes).
   Store as `ANDREA_LICENSE_SECRET` so the production binary embeds the
   same value as the licence-server CLI.
