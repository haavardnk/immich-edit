use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use url::Url;

use crate::error::AppError;
use crate::services::crypto::{Encrypted, InstanceCrypto};

pub const FLOW_COOKIE: &str = "immich_edit_oauth";
const FLOW_TTL_SECS: i64 = 600;
const STATE_LEN: usize = 32;
const VERIFIER_LEN: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FlowPurpose {
    Login,
    Setup,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthFlow {
    pub state: String,
    pub verifier: String,
    pub redirect_uri: String,
    pub next: Option<String>,
    pub immich_url: Option<String>,
    pub purpose: FlowPurpose,
    pub exp: i64,
}

impl OAuthFlow {
    pub fn begin(
        purpose: FlowPurpose,
        redirect_uri: &Url,
        next: Option<String>,
        immich_url: Option<String>,
    ) -> Self {
        Self {
            state: random_token(STATE_LEN),
            verifier: random_token(VERIFIER_LEN),
            redirect_uri: redirect_uri.as_str().to_string(),
            next,
            immich_url,
            purpose,
            exp: now_secs() + FLOW_TTL_SECS,
        }
    }

    pub fn challenge(&self) -> String {
        URL_SAFE_NO_PAD.encode(Sha256::digest(self.verifier.as_bytes()))
    }

    pub fn seal(&self, crypto: &InstanceCrypto) -> Result<String, AppError> {
        let plain = serde_json::to_vec(self).map_err(|_| AppError::Internal)?;
        let enc = crypto.encrypt(&plain).map_err(|_| AppError::Internal)?;
        let mut packed = Vec::with_capacity(1 + enc.nonce.len() + enc.ciphertext.len());
        packed.push(u8::try_from(enc.key_version).map_err(|_| AppError::Internal)?);
        packed.extend_from_slice(&enc.nonce);
        packed.extend_from_slice(&enc.ciphertext);
        Ok(URL_SAFE_NO_PAD.encode(packed))
    }

    pub fn open(crypto: &InstanceCrypto, sealed: &str) -> Result<Self, AppError> {
        let packed = URL_SAFE_NO_PAD
            .decode(sealed)
            .map_err(|_| AppError::BadRequest("oauth flow expired; start again".into()))?;
        if packed.len() < 13 {
            return Err(AppError::BadRequest(
                "oauth flow expired; start again".into(),
            ));
        }
        let enc = Encrypted {
            key_version: i64::from(packed[0]),
            nonce: packed[1..13].to_vec(),
            ciphertext: packed[13..].to_vec(),
        };
        let plain = crypto
            .decrypt(&enc)
            .map_err(|_| AppError::BadRequest("oauth flow expired; start again".into()))?;
        let flow: Self = serde_json::from_slice(plain.as_slice())
            .map_err(|_| AppError::BadRequest("oauth flow expired; start again".into()))?;
        if flow.exp <= now_secs() {
            return Err(AppError::BadRequest(
                "oauth flow expired; start again".into(),
            ));
        }
        Ok(flow)
    }
}

pub fn set_cookie(sealed: &str, secure: bool) -> String {
    let base = format!(
        "{FLOW_COOKIE}={sealed}; HttpOnly; SameSite=Lax; Path=/api; Max-Age={FLOW_TTL_SECS}"
    );
    if secure {
        format!("{base}; Secure")
    } else {
        base
    }
}

pub fn clear_cookie() -> String {
    format!("{FLOW_COOKIE}=; HttpOnly; SameSite=Lax; Path=/api; Max-Age=0")
}

pub fn read_cookie(raw: Option<&str>) -> Option<String> {
    let cookies = raw?;
    for pair in cookies.split(';') {
        if let Some(rest) = pair
            .trim()
            .strip_prefix(FLOW_COOKIE)
            .and_then(|r| r.strip_prefix('='))
        {
            return Some(rest.to_string());
        }
    }
    None
}

fn random_token(len: usize) -> String {
    let mut bytes = vec![0u8; len];
    rand::rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn crypto() -> Arc<InstanceCrypto> {
        let dir = tempfile::tempdir().unwrap().keep();
        Arc::new(InstanceCrypto::load_or_create(&dir.join("instance.key"), false).unwrap())
    }

    fn flow() -> OAuthFlow {
        let redirect = Url::parse("https://edit.example/login").unwrap();
        OAuthFlow::begin(FlowPurpose::Login, &redirect, Some("/albums".into()), None)
    }

    #[test]
    fn seal_then_open_roundtrips() {
        let c = crypto();
        let original = flow();
        let opened = OAuthFlow::open(&c, &original.seal(&c).unwrap()).unwrap();
        assert_eq!(opened.state, original.state);
        assert_eq!(opened.verifier, original.verifier);
        assert_eq!(opened.redirect_uri, "https://edit.example/login");
        assert_eq!(opened.next.as_deref(), Some("/albums"));
        assert_eq!(opened.purpose, FlowPurpose::Login);
    }

    #[test]
    fn a_tampered_cookie_is_rejected() {
        let c = crypto();
        let sealed = flow().seal(&c).unwrap();
        let mut bytes = URL_SAFE_NO_PAD.decode(&sealed).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 0x01;
        let tampered = URL_SAFE_NO_PAD.encode(&bytes);
        assert!(OAuthFlow::open(&c, &tampered).is_err());
    }

    #[test]
    fn another_instance_key_cannot_open_the_cookie() {
        let sealed = flow().seal(&crypto()).unwrap();
        assert!(OAuthFlow::open(&crypto(), &sealed).is_err());
    }

    #[test]
    fn an_expired_flow_is_rejected() {
        let c = crypto();
        let mut expired = flow();
        expired.exp = now_secs() - 1;
        assert!(OAuthFlow::open(&c, &expired.seal(&c).unwrap()).is_err());
    }

    #[test]
    fn the_challenge_is_the_s256_of_the_verifier() {
        let mut f = flow();
        f.verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk".into();
        assert_eq!(f.challenge(), "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM");
    }

    #[test]
    fn each_flow_gets_fresh_secrets() {
        let a = flow();
        let b = flow();
        assert_ne!(a.state, b.state);
        assert_ne!(a.verifier, b.verifier);
        assert_eq!(URL_SAFE_NO_PAD.decode(&a.state).unwrap().len(), STATE_LEN);
        assert_eq!(
            URL_SAFE_NO_PAD.decode(&a.verifier).unwrap().len(),
            VERIFIER_LEN
        );
    }

    #[test]
    fn the_cookie_is_lax_scoped_to_api() {
        let set = set_cookie("abc", true);
        assert!(set.contains("SameSite=Lax"));
        assert!(set.contains("Path=/api"));
        assert!(set.contains("HttpOnly"));
        assert!(set.contains("; Secure"));
        assert!(!set_cookie("abc", false).contains("Secure"));
    }

    #[test]
    fn read_cookie_finds_the_flow_among_others() {
        let header = "immich_edit_auth=session; immich_edit_oauth=sealed-value";
        assert_eq!(read_cookie(Some(header)).as_deref(), Some("sealed-value"));
        assert_eq!(read_cookie(Some("other=1")), None);
        assert_eq!(read_cookie(None), None);
    }
}
