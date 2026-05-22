use std::time::Duration;

use bytes::Bytes;
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use url::Url;
use uuid::Uuid;

use super::dto::{
    AlbumDetail, AlbumSummary, AssetDetail, AssetStatistics, BulkIdResponse, PeopleResponse,
    PersonSummary, SearchAssets, SearchResponse, TagSummary,
};
use super::{ImmichError, ImmichResult};

const API_KEY_HEADER: &str = "x-api-key";

#[derive(Debug, Clone)]
pub struct ImmichClient {
    http: Client,
    base: Url,
}

impl ImmichClient {
    pub fn new(base: Url, api_key: &str) -> ImmichResult<Self> {
        let mut headers = HeaderMap::new();
        let mut key_value = HeaderValue::from_str(api_key)
            .map_err(|_| ImmichError::Decode("invalid api key header".into()))?;
        key_value.set_sensitive(true);
        headers.insert(API_KEY_HEADER, key_value);
        headers.insert("accept", HeaderValue::from_static("application/json"));

        let http = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| ImmichError::Transport(e.to_string()))?;
        Ok(Self { http, base })
    }

    fn url(&self, path: &str) -> ImmichResult<Url> {
        let path = path.trim_start_matches('/');
        let mut base = self.base.clone();
        if !base.path().ends_with('/') {
            base.set_path(&format!("{}/", base.path()));
        }
        base.join(path)
            .map_err(|e| ImmichError::Decode(format!("url join: {e}")))
    }

    pub async fn ping(&self) -> ImmichResult<()> {
        let url = self.url("api/server/ping")?;
        send(&self.http, self.http.get(url)).await.map(|_| ())
    }

    pub async fn list_albums(&self) -> ImmichResult<Vec<AlbumSummary>> {
        self.get_json("api/albums").await
    }

    pub async fn album(&self, id: Uuid) -> ImmichResult<AlbumDetail> {
        self.get_json(&format!("api/albums/{id}")).await
    }

    pub async fn asset(&self, id: Uuid) -> ImmichResult<AssetDetail> {
        self.get_json(&format!("api/assets/{id}")).await
    }

    pub async fn thumbnail(&self, id: Uuid, size: ThumbSize) -> ImmichResult<(Bytes, String)> {
        let url = self.url(&format!("api/assets/{id}/thumbnail"))?;
        let req = self.http.get(url).query(&[("size", size.as_str())]);
        let resp = run(req).await?;
        let ct = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|h| h.to_str().ok())
            .unwrap_or("image/jpeg")
            .to_string();
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| ImmichError::Transport(e.to_string()))?;
        Ok((bytes, ct))
    }

    pub async fn original(&self, id: Uuid) -> ImmichResult<Bytes> {
        let url = self.url(&format!("api/assets/{id}/original"))?;
        send(&self.http, self.http.get(url)).await
    }

    pub async fn list_people(&self, named_only: bool) -> ImmichResult<Vec<PersonSummary>> {
        let url = self.url("api/people")?;
        let req = self
            .http
            .get(url)
            .query(&[("withHidden", "false"), ("size", "500")]);
        let bytes = send(&self.http, req).await?;
        let resp: PeopleResponse = parse_json(&bytes)?;
        let people = if named_only {
            resp.people
                .into_iter()
                .filter(|p| !p.name.is_empty())
                .collect()
        } else {
            resp.people
        };
        Ok(people)
    }

    pub async fn person_thumb(&self, id: Uuid) -> ImmichResult<(Bytes, String)> {
        let url = self.url(&format!("api/people/{id}/thumbnail"))?;
        let resp = run(self.http.get(url)).await?;
        let ct = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|h| h.to_str().ok())
            .unwrap_or("image/jpeg")
            .to_string();
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| ImmichError::Transport(e.to_string()))?;
        Ok((bytes, ct))
    }

    pub async fn list_tags(&self) -> ImmichResult<Vec<TagSummary>> {
        self.get_json("api/tags").await
    }

    pub async fn update_asset(
        &self,
        id: Uuid,
        body: &serde_json::Value,
    ) -> ImmichResult<AssetDetail> {
        let url = self.url(&format!("api/assets/{id}"))?;
        let bytes = send(&self.http, self.http.put(url).json(body)).await?;
        parse_json(&bytes)
    }

    pub async fn upsert_tags(&self, body: &serde_json::Value) -> ImmichResult<Vec<TagSummary>> {
        let url = self.url("api/tags")?;
        let bytes = send(&self.http, self.http.put(url).json(body)).await?;
        parse_json(&bytes)
    }

    pub async fn tag_asset(
        &self,
        tag_id: Uuid,
        asset_id: Uuid,
    ) -> ImmichResult<Vec<BulkIdResponse>> {
        let url = self.url(&format!("api/tags/{tag_id}/assets"))?;
        let body = serde_json::json!({ "ids": [asset_id] });
        let bytes = send(&self.http, self.http.put(url).json(&body)).await?;
        parse_json(&bytes)
    }

    pub async fn untag_asset(
        &self,
        tag_id: Uuid,
        asset_id: Uuid,
    ) -> ImmichResult<Vec<BulkIdResponse>> {
        let url = self.url(&format!("api/tags/{tag_id}/assets"))?;
        let body = serde_json::json!({ "ids": [asset_id] });
        let bytes = send(&self.http, self.http.delete(url).json(&body)).await?;
        parse_json(&bytes)
    }

    pub async fn folder_paths(&self) -> ImmichResult<Vec<String>> {
        self.get_json("api/view/folder/unique-paths").await
    }

    pub async fn folder_assets(&self, path: &str) -> ImmichResult<Vec<AssetDetail>> {
        let url = self.url("api/view/folder")?;
        let req = self.http.get(url).query(&[("path", path)]);
        let bytes = send(&self.http, req).await?;
        parse_json(&bytes)
    }

    pub async fn search_metadata(&self, body: &serde_json::Value) -> ImmichResult<SearchAssets> {
        let url = self.url("api/search/metadata")?;
        let bytes = send_post_json(&self.http, url, body).await?;
        let resp: SearchResponse = parse_json(&bytes)?;
        Ok(resp.assets)
    }

    pub async fn asset_statistics(
        &self,
        query: &[(String, String)],
    ) -> ImmichResult<AssetStatistics> {
        let mut url = self.url("api/assets/statistics")?;
        for (k, v) in query {
            url.query_pairs_mut().append_pair(k, v);
        }
        let bytes = send(&self.http, self.http.get(url)).await?;
        parse_json(&bytes)
    }

    async fn get_json<T: serde::de::DeserializeOwned>(&self, path: &str) -> ImmichResult<T> {
        let url = self.url(path)?;
        let bytes = send(&self.http, self.http.get(url)).await?;
        parse_json(&bytes)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ThumbSize {
    Thumbnail,
    Preview,
}

impl ThumbSize {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Thumbnail => "thumbnail",
            Self::Preview => "preview",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "thumbnail" => Some(Self::Thumbnail),
            "preview" => Some(Self::Preview),
            _ => None,
        }
    }
}

async fn send(_http: &Client, req: reqwest::RequestBuilder) -> ImmichResult<Bytes> {
    let resp = run(req).await?;
    resp.bytes()
        .await
        .map_err(|e| ImmichError::Transport(e.to_string()))
}

fn parse_json<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> ImmichResult<T> {
    serde_json::from_slice(bytes).map_err(|e| ImmichError::Decode(e.to_string()))
}

async fn send_post_json(http: &Client, url: Url, body: &serde_json::Value) -> ImmichResult<Bytes> {
    let req = http.post(url).json(body);
    let resp = run(req).await?;
    resp.bytes()
        .await
        .map_err(|e| ImmichError::Transport(e.to_string()))
}

async fn run(req: reqwest::RequestBuilder) -> ImmichResult<reqwest::Response> {
    let resp = req.send().await.map_err(map_send_err)?;
    let status = resp.status();
    if status.is_success() {
        return Ok(resp);
    }
    Err(match status.as_u16() {
        401 | 403 => ImmichError::Unauthorized,
        404 => ImmichError::NotFound,
        408 => ImmichError::Timeout,
        code => ImmichError::Status(code),
    })
}

fn map_send_err(err: reqwest::Error) -> ImmichError {
    if err.is_timeout() {
        ImmichError::Timeout
    } else {
        ImmichError::Transport(err.without_url().to_string())
    }
}
