use bytes::Bytes;
use futures_util::future::try_join_all;
use reqwest::header::{CONTENT_RANGE, ETAG, HeaderValue, LAST_MODIFIED, RANGE};
use reqwest::{RequestBuilder, Response, StatusCode};

use super::client::run_idempotent;
use super::{ImmichError, ImmichResult};

const MAX_ORIGINAL_BYTES: u64 = 2 << 30;

#[derive(Clone, Copy)]
pub(super) struct Split {
    pub head: u64,
    pub min_part: u64,
    pub tail_parts: u64,
}

impl Split {
    pub const DEFAULT: Self = Self {
        head: 8 << 20,
        min_part: 4 << 20,
        tail_parts: 3,
    };
}

pub(super) async fn download(
    request: impl Fn() -> RequestBuilder,
    split: Split,
) -> ImmichResult<Bytes> {
    let head = run_idempotent(request().header(RANGE, range(0, split.head))).await?;
    if head.status() != StatusCode::PARTIAL_CONTENT {
        return whole(head).await;
    }
    let (start, end, total) = content_range(&head)?;
    if start != 0 || end + 1 != total.min(split.head) {
        return Err(ImmichError::Decode(format!(
            "original range {start}-{end}/{total} does not match the request"
        )));
    }
    if total <= split.head {
        return whole(head).await;
    }
    if total > MAX_ORIGINAL_BYTES {
        return Err(ImmichError::Decode(format!("original is {total} bytes")));
    }
    let version = version(&head);
    let mut buf = vec![0u8; total as usize];
    let (head_buf, rest) = buf.split_at_mut(split.head as usize);
    let parts = (rest.len() as u64)
        .div_ceil(split.min_part)
        .clamp(1, split.tail_parts);
    let part_len = rest.len().div_ceil(parts as usize);
    let tails = rest.chunks_mut(part_len).enumerate().map(|(i, slice)| {
        let start = split.head + (i * part_len) as u64;
        tail(&request, start, total, version.as_ref(), slice)
    });
    tokio::try_join!(fill(head, head_buf), try_join_all(tails))?;
    Ok(Bytes::from(buf))
}

async fn tail(
    request: &impl Fn() -> RequestBuilder,
    start: u64,
    total: u64,
    expected: Option<&HeaderValue>,
    slice: &mut [u8],
) -> ImmichResult<()> {
    let end = start + slice.len() as u64;
    let resp = run_idempotent(request().header(RANGE, range(start, end))).await?;
    if resp.status() != StatusCode::PARTIAL_CONTENT
        || content_range(&resp)? != (start, end - 1, total)
        || version(&resp).as_ref() != expected
    {
        return Err(ImmichError::Transport(
            "original changed during a ranged download".into(),
        ));
    }
    fill(resp, slice).await
}

async fn whole(resp: Response) -> ImmichResult<Bytes> {
    resp.bytes()
        .await
        .map_err(|e| ImmichError::Transport(e.to_string()))
}

async fn fill(mut resp: Response, slice: &mut [u8]) -> ImmichResult<()> {
    let mut at = 0;
    while let Some(chunk) = resp
        .chunk()
        .await
        .map_err(|e| ImmichError::Transport(e.to_string()))?
    {
        let end = at + chunk.len();
        let Some(dst) = slice.get_mut(at..end) else {
            return Err(ImmichError::Transport("original range ran long".into()));
        };
        dst.copy_from_slice(&chunk);
        at = end;
    }
    if at != slice.len() {
        return Err(ImmichError::Transport("original range ended early".into()));
    }
    Ok(())
}

fn range(start: u64, end: u64) -> String {
    format!("bytes={start}-{}", end - 1)
}

fn content_range(resp: &Response) -> ImmichResult<(u64, u64, u64)> {
    let invalid = || ImmichError::Decode("invalid content-range".into());
    let value = resp
        .headers()
        .get(CONTENT_RANGE)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(invalid)?;
    let (span, total) = value
        .strip_prefix("bytes ")
        .and_then(|v| v.split_once('/'))
        .ok_or_else(invalid)?;
    let (start, end) = span.split_once('-').ok_or_else(invalid)?;
    let parse = |s: &str| s.parse::<u64>().map_err(|_| invalid());
    Ok((parse(start)?, parse(end)?, parse(total)?))
}

fn version(resp: &Response) -> Option<HeaderValue> {
    resp.headers()
        .get(ETAG)
        .or_else(|| resp.headers().get(LAST_MODIFIED))
        .cloned()
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use wiremock::matchers::method;
    use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

    use super::*;

    const SPLIT: Split = Split {
        head: 1_000,
        min_part: 1_000,
        tail_parts: 3,
    };

    #[derive(Clone, Copy, Debug)]
    enum Server {
        Ranged,
        IgnoresRange,
        Changing,
    }

    struct Original {
        body: Vec<u8>,
        server: Server,
        served: AtomicUsize,
    }

    impl Respond for Original {
        fn respond(&self, request: &Request) -> ResponseTemplate {
            let served = self.served.fetch_add(1, Ordering::Relaxed);
            let len = self.body.len();
            let requested = request
                .headers
                .get("range")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("bytes="))
                .and_then(|v| v.split_once('-'))
                .map(|(a, b)| (a.parse::<usize>().unwrap(), b.parse::<usize>().unwrap()));
            let etag = match self.server {
                Server::Changing => format!("\"v{served}\""),
                _ => "\"v0\"".into(),
            };
            let Some((start, end)) =
                requested.filter(|_| !matches!(self.server, Server::IgnoresRange))
            else {
                return ResponseTemplate::new(200).set_body_bytes(self.body.clone());
            };
            let end = end.min(len - 1);
            ResponseTemplate::new(206)
                .insert_header("content-range", format!("bytes {start}-{end}/{len}"))
                .insert_header("etag", etag)
                .set_body_bytes(self.body[start..=end].to_vec())
        }
    }

    async fn fetch(server: Server, len: usize) -> (ImmichResult<Bytes>, Vec<u8>, usize) {
        let body: Vec<u8> = (0..len).map(|i| (i * 31 % 251) as u8).collect();
        let mock = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(Original {
                body: body.clone(),
                server,
                served: AtomicUsize::new(0),
            })
            .mount(&mock)
            .await;
        let http = reqwest::Client::new();
        let url = format!("{}/original", mock.uri());
        let result = download(|| http.get(&url), SPLIT).await;
        let requests = mock.received_requests().await.unwrap_or_default().len();
        (result, body, requests)
    }

    #[tokio::test]
    async fn originals_arrive_whole_in_as_many_ranges_as_their_size_allows() {
        let cases = [
            (Server::Ranged, 10_000, 4),
            (Server::Ranged, 2_500, 3),
            (Server::Ranged, 600, 1),
            (Server::IgnoresRange, 10_000, 1),
        ];
        for (server, len, expected) in cases {
            let (result, body, requests) = fetch(server, len).await;
            let Ok(bytes) = result else {
                panic!("{server:?} {len}: {result:?}");
            };
            if bytes.as_ref() != body.as_slice() || requests != expected {
                panic!(
                    "{server:?} {len}: {} bytes in {requests} requests, wanted {expected}",
                    bytes.len()
                );
            }
        }
    }

    #[tokio::test]
    async fn an_original_replaced_mid_download_is_rejected() {
        let (result, _, _) = fetch(Server::Changing, 10_000).await;
        if result.is_ok() {
            panic!("mixed versions were stitched together");
        }
    }
}
