//! Streaming, resumable, checksummed file downloader.
//!
//! Algorithm:
//!
//! 1. Stat the destination file. If it exists, compute its size and reuse it
//!    as the resume offset; pass that offset in a `Range: bytes=N-` header.
//! 2. Stream the response body in chunks. Each chunk is appended to disk
//!    *and* fed into a running SHA-256 hasher so we never re-read the file
//!    after the download completes.
//! 3. On completion, compare the final digest with the expected hex. If
//!    it mismatches, delete the file and surface a clear error.
//! 4. If the server does not honour the `Range` header (e.g. ignores it and
//!    sends 200 OK), restart from zero — this is detected by inspecting the
//!    HTTP status and the `Content-Range` header.
//!
//! Progress is reported through a callback closure that receives a
//! [`DownloadProgress`] struct after every chunk.

use std::path::Path;

use bytes::Bytes;
use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, SeekFrom};

/// Progress event emitted by [`download_with_resume`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DownloadProgress {
    /// Bytes downloaded so far (across all sessions).
    pub downloaded: u64,
    /// Expected total in bytes. May be `None` if the server did not return a
    /// `Content-Length`.
    pub total: Option<u64>,
}

impl DownloadProgress {
    /// Fractional progress in `[0.0, 1.0]` when `total` is known.
    pub fn fraction(&self) -> Option<f32> {
        match self.total {
            Some(t) if t > 0 => Some((self.downloaded as f32 / t as f32).clamp(0.0, 1.0)),
            _ => None,
        }
    }
}

/// Errors raised by the downloader.
#[derive(Debug, Error)]
pub enum DownloadError {
    /// Network or HTTP-level failure.
    #[error("download transport error: {0}")]
    Transport(String),
    /// Server returned an unexpected status code.
    #[error("download HTTP error: status {status}")]
    Status {
        /// HTTP status code returned.
        status: u16,
    },
    /// SHA-256 mismatch after the file finished downloading.
    #[error("checksum mismatch: expected {expected}, got {actual}")]
    Checksum {
        /// Expected SHA-256 (lowercase hex).
        expected: String,
        /// Actual SHA-256 (lowercase hex).
        actual: String,
    },
    /// Local I/O failure.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// The on-disk file is larger than the expected size — refusing to resume.
    #[error("local file is larger than expected ({local} > {expected}); delete to retry")]
    LocalFileTooLarge {
        /// Size on disk.
        local: u64,
        /// Expected total size.
        expected: u64,
    },
}

impl From<reqwest::Error> for DownloadError {
    fn from(e: reqwest::Error) -> Self {
        DownloadError::Transport(e.to_string())
    }
}

/// Download `url` to `dest`, resuming any partial file already present.
///
/// `expected_sha256_hex` is the lowercase hex digest of the entire final
/// file. `expected_size_bytes` is informational (used to bound the resume
/// offset). Pass `0` to disable that check.
///
/// `on_progress` is called after every received chunk.
pub async fn download_with_resume<F>(
    client: &reqwest::Client,
    url: &str,
    dest: &Path,
    expected_sha256_hex: &str,
    expected_size_bytes: u64,
    mut on_progress: F,
) -> Result<(), DownloadError>
where
    F: FnMut(DownloadProgress),
{
    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let mut resume_from: u64 = match tokio::fs::metadata(dest).await {
        Ok(meta) if meta.is_file() => meta.len(),
        _ => 0,
    };

    if expected_size_bytes > 0 && resume_from > expected_size_bytes {
        return Err(DownloadError::LocalFileTooLarge {
            local: resume_from,
            expected: expected_size_bytes,
        });
    }

    // Build the request, optionally with Range.
    let mut req = client.get(url);
    if resume_from > 0 {
        req = req.header("Range", format!("bytes={resume_from}-"));
    }
    let resp = req.send().await?;
    let status = resp.status();
    if status == reqwest::StatusCode::OK {
        // Server ignored Range or we asked for the whole file.
        // Truncate any pre-existing partial download.
        resume_from = 0;
    } else if status != reqwest::StatusCode::PARTIAL_CONTENT && !status.is_success() {
        return Err(DownloadError::Status {
            status: status.as_u16(),
        });
    }

    let total = total_size_from_response(&resp, resume_from);
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(resume_from == 0)
        .open(dest)
        .await?;
    if resume_from > 0 {
        file.seek(SeekFrom::Start(resume_from)).await?;
    }

    // Hash what we already have on disk so we can verify the final digest
    // without re-reading.
    let mut hasher = Sha256::new();
    if resume_from > 0 {
        hash_existing_prefix(&mut hasher, dest, resume_from).await?;
    }

    let mut downloaded: u64 = resume_from;
    on_progress(DownloadProgress { downloaded, total });

    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk: Bytes = chunk?;
        if chunk.is_empty() {
            continue;
        }
        hasher.update(&chunk);
        file.write_all(&chunk).await?;
        downloaded = downloaded.saturating_add(chunk.len() as u64);
        on_progress(DownloadProgress { downloaded, total });
    }
    file.flush().await?;
    file.shutdown().await?;
    drop(file);

    let actual_hex = hex::encode(hasher.finalize());
    if !expected_sha256_hex.is_empty() && !actual_hex.eq_ignore_ascii_case(expected_sha256_hex) {
        // Remove the corrupted file so the next attempt starts clean.
        let _ = tokio::fs::remove_file(dest).await;
        return Err(DownloadError::Checksum {
            expected: expected_sha256_hex.to_lowercase(),
            actual: actual_hex,
        });
    }
    Ok(())
}

fn total_size_from_response(resp: &reqwest::Response, resume_from: u64) -> Option<u64> {
    if let Some(value) = resp.headers().get(reqwest::header::CONTENT_RANGE) {
        // Format: "bytes start-end/total"
        if let Ok(text) = value.to_str() {
            if let Some(slash) = text.find('/') {
                if let Ok(total) = text[slash + 1..].trim().parse::<u64>() {
                    return Some(total);
                }
            }
        }
    }
    if let Some(value) = resp.headers().get(reqwest::header::CONTENT_LENGTH) {
        if let Ok(text) = value.to_str() {
            if let Ok(content_len) = text.trim().parse::<u64>() {
                return Some(resume_from.saturating_add(content_len));
            }
        }
    }
    None
}

async fn hash_existing_prefix(
    hasher: &mut Sha256,
    path: &Path,
    bytes_to_hash: u64,
) -> Result<(), DownloadError> {
    let mut file = File::open(path).await?;
    let mut remaining = bytes_to_hash;
    let mut buf = vec![0u8; 64 * 1024];
    while remaining > 0 {
        let to_read = remaining.min(buf.len() as u64) as usize;
        let n = file.read(&mut buf[..to_read]).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        remaining -= n as u64;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::Digest;
    use wiremock::matchers::{header_exists, method, path};
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    fn sha256_hex(bytes: &[u8]) -> String {
        let mut h = Sha256::new();
        h.update(bytes);
        hex::encode(h.finalize())
    }

    #[tokio::test]
    async fn fresh_download_writes_file_and_verifies_checksum() {
        let payload: Vec<u8> = (0..200_000u32).map(|i| (i % 251) as u8).collect();
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/asset.bin"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_bytes(payload.clone())
                    .insert_header("Content-Length", payload.len().to_string()),
            )
            .mount(&server)
            .await;

        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("asset.bin");
        let url = format!("{}/asset.bin", server.uri());

        let mut last = DownloadProgress {
            downloaded: 0,
            total: None,
        };
        let client = reqwest::Client::new();
        download_with_resume(
            &client,
            &url,
            &dest,
            &sha256_hex(&payload),
            payload.len() as u64,
            |p| last = p,
        )
        .await
        .unwrap();

        assert_eq!(tokio::fs::read(&dest).await.unwrap(), payload);
        assert_eq!(last.downloaded, payload.len() as u64);
        assert_eq!(last.total, Some(payload.len() as u64));
    }

    #[tokio::test]
    async fn resumes_when_partial_file_already_on_disk() {
        let payload: Vec<u8> = (0..120_000u32).map(|i| (i % 251) as u8).collect();
        let split = 50_000;
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("asset.bin");
        tokio::fs::write(&dest, &payload[..split]).await.unwrap();

        let server = MockServer::start().await;
        let payload_clone = payload.clone();
        Mock::given(method("GET"))
            .and(path("/asset.bin"))
            .and(header_exists("Range"))
            .respond_with(move |req: &Request| {
                let range = req
                    .headers
                    .get("Range")
                    .map(|v| v.to_str().unwrap_or(""))
                    .unwrap_or("");
                let start: u64 = range
                    .strip_prefix("bytes=")
                    .and_then(|s| s.split('-').next())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let body = payload_clone[start as usize..].to_vec();
                let total = payload_clone.len() as u64;
                let end = total - 1;
                ResponseTemplate::new(206)
                    .set_body_bytes(body)
                    .insert_header("Content-Range", format!("bytes {start}-{end}/{total}"))
            })
            .mount(&server)
            .await;

        let url = format!("{}/asset.bin", server.uri());
        let client = reqwest::Client::new();
        let mut progress_events = 0;
        download_with_resume(
            &client,
            &url,
            &dest,
            &sha256_hex(&payload),
            payload.len() as u64,
            |_| progress_events += 1,
        )
        .await
        .unwrap();

        assert_eq!(tokio::fs::read(&dest).await.unwrap(), payload);
        assert!(progress_events >= 2);
    }

    #[tokio::test]
    async fn checksum_mismatch_deletes_file() {
        let payload = b"hello world".to_vec();
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/asset.bin"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(payload.clone()))
            .mount(&server)
            .await;

        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("asset.bin");
        let url = format!("{}/asset.bin", server.uri());
        let client = reqwest::Client::new();

        let bogus_hex = "0".repeat(64);
        let err = download_with_resume(
            &client,
            &url,
            &dest,
            &bogus_hex,
            payload.len() as u64,
            |_| {},
        )
        .await
        .unwrap_err();

        match err {
            DownloadError::Checksum { expected, actual } => {
                assert_eq!(expected, bogus_hex);
                assert_eq!(actual, sha256_hex(&payload));
            }
            other => panic!("expected Checksum, got {other:?}"),
        }
        assert!(!dest.exists());
    }

    #[tokio::test]
    async fn http_error_is_surfaced() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/asset.bin"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("asset.bin");
        let url = format!("{}/asset.bin", server.uri());
        let client = reqwest::Client::new();
        let err = download_with_resume(&client, &url, &dest, "", 0, |_| {})
            .await
            .unwrap_err();
        assert!(matches!(err, DownloadError::Status { status: 404 }));
    }

    #[tokio::test]
    async fn local_file_larger_than_expected_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("asset.bin");
        tokio::fs::write(&dest, vec![0u8; 200]).await.unwrap();
        let client = reqwest::Client::new();
        let err = download_with_resume(&client, "http://127.0.0.1:1/x", &dest, "", 100, |_| {})
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            DownloadError::LocalFileTooLarge {
                local: 200,
                expected: 100
            }
        ));
    }

    #[tokio::test]
    async fn server_ignoring_range_restarts_from_zero() {
        let payload: Vec<u8> = (0..50_000u32).map(|i| (i % 251) as u8).collect();
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("asset.bin");
        tokio::fs::write(&dest, &payload[..10_000]).await.unwrap();

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/asset.bin"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(payload.clone()))
            .mount(&server)
            .await;

        let url = format!("{}/asset.bin", server.uri());
        let client = reqwest::Client::new();
        download_with_resume(&client, &url, &dest, &sha256_hex(&payload), 0, |_| {})
            .await
            .unwrap();
        assert_eq!(tokio::fs::read(&dest).await.unwrap(), payload);
    }

    #[test]
    fn fraction_is_clamped_and_optional() {
        let p = DownloadProgress {
            downloaded: 50,
            total: Some(100),
        };
        assert!((p.fraction().unwrap() - 0.5).abs() < f32::EPSILON);
        let p = DownloadProgress {
            downloaded: 200,
            total: Some(100),
        };
        assert!((p.fraction().unwrap() - 1.0).abs() < f32::EPSILON);
        let p = DownloadProgress {
            downloaded: 50,
            total: None,
        };
        assert!(p.fraction().is_none());
    }
}
