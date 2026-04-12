//! Download tool with resume support - B-W12/02

use super::ToolError;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_LENGTH, RANGE};

use std::path::Path;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

const CHUNK_SIZE: usize = 8192;

pub struct DownloadOptions<F: Fn(u64, u64)> {
    pub resume: bool,
    pub on_progress: F,
}

impl<F: Fn(u64, u64)> DownloadOptions<F> {
    pub fn new(on_progress: F) -> Self {
        Self { resume: true, on_progress }
    }
}

pub async fn download_file<P: AsRef<Path>, F: Fn(u64, u64)>(
    url: &str,
    dest: P,
    opts: DownloadOptions<F>,
) -> Result<u64, ToolError> {
    let dest = dest.as_ref();
    let start_pos = if opts.resume && dest.exists() {
        tokio::fs::metadata(dest).await.map(|m| m.len()).unwrap_or(0)
    } else { 0 };

    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    if start_pos > 0 {
        headers.insert(RANGE, HeaderValue::from_str(&format!("bytes={}-", start_pos))
            .map_err(|e| ToolError::new(format!("Range header: {}", e)))?);
    }

    let resp = client.get(url).headers(headers).send().await
        .map_err(|e| ToolError::new(format!("Request: {}", e)))?;
    let total = resp.headers().get(CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .map(|l| if resp.status() == reqwest::StatusCode::PARTIAL_CONTENT { start_pos + l } else { l })
        .unwrap_or(start_pos);

    let mut file = OpenOptions::new().create(true).append(true).open(dest).await
        .map_err(|e| ToolError::new(format!("Open: {}", e)))?;
    let mut stream = resp.bytes_stream();
    let mut downloaded = start_pos;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| ToolError::new(format!("Stream: {}", e)))?;
        file.write_all(&chunk).await.map_err(|e| {
            if e.raw_os_error() == Some(112) { ToolError::disk_full("Disk full") }
            else { ToolError::new(format!("Write: {}", e)) }
        })?;
        downloaded += chunk.len() as u64;
        (opts.on_progress)(downloaded, total);
    }
    file.flush().await.map_err(|e| ToolError::new(format!("Flush: {}", e)))?;
    Ok(downloaded)
}

use futures::StreamExt;

pub async fn download_simple(url: &str, dest: &Path) -> Result<(), ToolError> {
    download_file(url, dest, DownloadOptions::new(|_, _| {})).await.map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_chunk_size() { assert_eq!(CHUNK_SIZE, 8192); }
}
