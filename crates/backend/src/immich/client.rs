use std::time::Duration;

use bytes::Bytes;
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use url::Url;
use uuid::Uuid;

use super::dto::{
    AlbumDetail, AlbumSummary, AssetDetail, BulkIdResponse, PeopleResponse, PersonSummary,
    SearchAssets, SearchResponse, SearchStatistics, StackDetail, TagSummary, UploadResponse,
};
use super::{ImmichError, ImmichResult};

const API_KEY_HEADER: &str = "x-api-key";

#[derive(Clone)]
pub enum ImmichAuth {
    ApiKey(String),
    Bearer(String),
}

impl std::fmt::Debug for ImmichAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ApiKey(_) => f.write_str("ApiKey(***)"),
            Self::Bearer(_) => f.write_str("Bearer(***)"),
        }
    }
}

impl ImmichAuth {
    fn header(&self) -> ImmichResult<(HeaderName, HeaderValue)> {
        let (name, value_str) = match self {
            Self::ApiKey(k) => (HeaderName::from_static(API_KEY_HEADER), k.clone()),
            Self::Bearer(t) => (reqwest::header::AUTHORIZATION, format!("Bearer {t}")),
        };
        let mut value = HeaderValue::from_str(&value_str)
            .map_err(|_| ImmichError::Decode("invalid auth header".into()))?;
        value.set_sensitive(true);
        Ok((name, value))
    }
}

#[derive(Debug, Clone)]
pub struct ImmichClient {
    http: Client,
    base: Url,
    auth_name: HeaderName,
    auth_value: HeaderValue,
}

impl ImmichClient {
    pub fn with_auth(base: Url, auth: ImmichAuth, request_timeout: Duration) -> ImmichResult<Self> {
        let mut headers = HeaderMap::new();
        headers.insert("accept", HeaderValue::from_static("application/json"));
        let http = Client::builder()
            .default_headers(headers)
            .timeout(request_timeout)
            .connect_timeout(Duration::from_secs(5))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| ImmichError::Transport(e.to_string()))?;
        let (auth_name, auth_value) = auth.header()?;
        Ok(Self {
            http,
            base,
            auth_name,
            auth_value,
        })
    }

    fn authed(&self, rb: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        rb.header(self.auth_name.clone(), self.auth_value.clone())
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
        send(self.authed(self.http.get(url))).await.map(|_| ())
    }

    pub async fn login_password(&self, email: &str, password: &str) -> ImmichResult<ImmichLogin> {
        let url = self.url("api/auth/login")?;
        let body = serde_json::json!({ "email": email, "password": password });
        let bytes = send_post_json(self.http.post(url).json(&body)).await?;
        parse_json(&bytes)
    }

    pub async fn me(&self) -> ImmichResult<ImmichUser> {
        self.get_json("api/users/me").await
    }

    pub async fn logout(&self) -> ImmichResult<()> {
        let url = self.url("api/auth/logout")?;
        send(self.authed(self.http.post(url))).await.map(|_| ())
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
        let req = self.authed(self.http.get(url).query(&[("size", size.as_str())]));
        self.bytes_with_content_type(req).await
    }

    pub async fn original(&self, id: Uuid) -> ImmichResult<Bytes> {
        let url = self.url(&format!("api/assets/{id}/original"))?;
        send(self.authed(self.http.get(url))).await
    }

    pub async fn list_people(&self, named_only: bool) -> ImmichResult<Vec<PersonSummary>> {
        let url = self.url("api/people")?;
        let req = self.authed(
            self.http
                .get(url)
                .query(&[("withHidden", "false"), ("size", "500")]),
        );
        let bytes = send(req).await?;
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
        self.bytes_with_content_type(self.authed(self.http.get(url)))
            .await
    }

    pub async fn list_tags(&self) -> ImmichResult<Vec<TagSummary>> {
        self.get_json("api/tags").await
    }

    pub async fn update_asset(
        &self,
        id: Uuid,
        body: &serde_json::Value,
    ) -> ImmichResult<AssetDetail> {
        self.put_json(&format!("api/assets/{id}"), body).await
    }

    pub async fn upsert_tags(&self, body: &serde_json::Value) -> ImmichResult<Vec<TagSummary>> {
        self.put_json("api/tags", body).await
    }

    pub async fn set_asset_tag(
        &self,
        tag_id: Uuid,
        asset_id: Uuid,
        attach: bool,
    ) -> ImmichResult<Vec<BulkIdResponse>> {
        let path = format!("api/tags/{tag_id}/assets");
        let body = serde_json::json!({ "ids": [asset_id] });
        if attach {
            self.put_json(&path, &body).await
        } else {
            self.delete_json(&path, &body).await
        }
    }

    pub async fn upload_asset(&self, req: UploadRequest<'_>) -> ImmichResult<UploadResponse> {
        let url = self.url("api/assets")?;
        let part = reqwest::multipart::Part::bytes(req.bytes.to_vec())
            .file_name(req.filename.to_string())
            .mime_str(req.content_type)
            .map_err(|e| ImmichError::Decode(format!("mime: {e}")))?;
        let form = reqwest::multipart::Form::new()
            .text("deviceAssetId", req.device_asset_id.to_string())
            .text("deviceId", "immich-edit".to_string())
            .text("filename", req.filename.to_string())
            .text("fileCreatedAt", req.created_at.to_string())
            .text("fileModifiedAt", req.modified_at.to_string())
            .text("isFavorite", req.is_favorite.to_string())
            .part("assetData", part);
        let bytes = send(self.authed(self.http.post(url).multipart(form))).await?;
        parse_json(&bytes)
    }

    pub async fn add_assets_to_album(
        &self,
        album_id: Uuid,
        asset_ids: &[Uuid],
    ) -> ImmichResult<Vec<BulkIdResponse>> {
        let body = serde_json::json!({ "ids": asset_ids });
        self.put_json(&format!("api/albums/{album_id}/assets"), &body)
            .await
    }

    pub async fn get_stack(&self, id: Uuid) -> ImmichResult<StackDetail> {
        self.get_json(&format!("api/stacks/{id}")).await
    }

    pub async fn create_stack(&self, asset_ids: &[Uuid]) -> ImmichResult<StackDetail> {
        let url = self.url("api/stacks")?;
        let body = serde_json::json!({ "assetIds": asset_ids });
        let bytes = send(self.authed(self.http.post(url).json(&body))).await?;
        parse_json(&bytes)
    }

    pub async fn update_stack_primary(
        &self,
        stack_id: Uuid,
        primary_asset_id: Uuid,
    ) -> ImmichResult<StackDetail> {
        let body = serde_json::json!({ "primaryAssetId": primary_asset_id });
        self.put_json(&format!("api/stacks/{stack_id}"), &body)
            .await
    }

    pub async fn folder_paths(&self) -> ImmichResult<Vec<String>> {
        self.get_json("api/view/folder/unique-paths").await
    }

    pub async fn folder_assets(&self, path: &str) -> ImmichResult<Vec<AssetDetail>> {
        let url = self.url("api/view/folder")?;
        let req = self.authed(self.http.get(url).query(&[("path", path)]));
        let bytes = send(req).await?;
        parse_json(&bytes)
    }

    pub async fn search_metadata(&self, body: &serde_json::Value) -> ImmichResult<SearchAssets> {
        let resp: SearchResponse = self.post_json("api/search/metadata", body).await?;
        Ok(resp.assets)
    }

    pub async fn search_smart(&self, body: &serde_json::Value) -> ImmichResult<SearchAssets> {
        let resp: SearchResponse = self.post_json("api/search/smart", body).await?;
        Ok(resp.assets)
    }

    pub async fn search_statistics(
        &self,
        body: &serde_json::Value,
    ) -> ImmichResult<SearchStatistics> {
        self.post_json("api/search/statistics", body).await
    }

    async fn get_json<T: serde::de::DeserializeOwned>(&self, path: &str) -> ImmichResult<T> {
        let url = self.url(path)?;
        let bytes = send(self.authed(self.http.get(url))).await?;
        parse_json(&bytes)
    }

    async fn put_json<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> ImmichResult<T> {
        let url = self.url(path)?;
        let bytes = send(self.authed(self.http.put(url).json(body))).await?;
        parse_json(&bytes)
    }

    async fn delete_json<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> ImmichResult<T> {
        let url = self.url(path)?;
        let bytes = send(self.authed(self.http.delete(url).json(body))).await?;
        parse_json(&bytes)
    }

    async fn post_json<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> ImmichResult<T> {
        let url = self.url(path)?;
        let bytes = send_post_json(self.authed(self.http.post(url).json(body))).await?;
        parse_json(&bytes)
    }

    async fn bytes_with_content_type(
        &self,
        req: reqwest::RequestBuilder,
    ) -> ImmichResult<(Bytes, String)> {
        let resp = run(req).await?;
        let content_type = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|h| h.to_str().ok())
            .unwrap_or("image/jpeg")
            .to_string();
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| ImmichError::Transport(e.to_string()))?;
        Ok((bytes, content_type))
    }
}

pub struct UploadRequest<'a> {
    pub filename: &'a str,
    pub content_type: &'a str,
    pub bytes: Bytes,
    pub is_favorite: bool,
    pub created_at: &'a str,
    pub modified_at: &'a str,
    pub device_asset_id: &'a str,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImmichLogin {
    pub access_token: String,
    pub user_id: Uuid,
    pub user_email: String,
    pub name: String,
    pub is_admin: bool,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImmichUser {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub is_admin: bool,
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

async fn send(req: reqwest::RequestBuilder) -> ImmichResult<Bytes> {
    let resp = run_idempotent(req).await?;
    resp.bytes()
        .await
        .map_err(|e| ImmichError::Transport(e.to_string()))
}

fn parse_json<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> ImmichResult<T> {
    serde_json::from_slice(bytes).map_err(|e| ImmichError::Decode(e.to_string()))
}

async fn send_post_json(req: reqwest::RequestBuilder) -> ImmichResult<Bytes> {
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

async fn run_idempotent(req: reqwest::RequestBuilder) -> ImmichResult<reqwest::Response> {
    const ATTEMPTS: u32 = 3;
    let mut last: Option<ImmichError> = None;
    for attempt in 0..ATTEMPTS {
        let try_req = match req.try_clone() {
            Some(r) => r,
            None => return run(req).await,
        };
        match run(try_req).await {
            Ok(resp) => return Ok(resp),
            Err(err) if is_retryable(&err) && attempt + 1 < ATTEMPTS => {
                last = Some(err);
                let base_ms = 100u64 << attempt;
                let jitter_ms = jitter_ms(base_ms);
                tokio::time::sleep(std::time::Duration::from_millis(base_ms + jitter_ms)).await;
            }
            Err(err) => return Err(err),
        }
    }
    Err(last.unwrap_or(ImmichError::Transport("retry exhausted".into())))
}

fn is_retryable(err: &ImmichError) -> bool {
    matches!(
        err,
        ImmichError::Status(502..=504) | ImmichError::Transport(_) | ImmichError::Timeout
    )
}

fn jitter_ms(base: u64) -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(0);
    nanos % base.max(1)
}

fn map_send_err(err: reqwest::Error) -> ImmichError {
    if err.is_timeout() {
        ImmichError::Timeout
    } else {
        ImmichError::Transport(err.without_url().to_string())
    }
}
