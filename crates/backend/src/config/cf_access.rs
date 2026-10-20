use reqwest::header::{HeaderMap, HeaderValue};

use super::ConfigError;

const ID_HEADER: &str = "cf-access-client-id";
const SECRET_HEADER: &str = "cf-access-client-secret";
const ID_KEY: &str = "CF_ACCESS_CLIENT_ID";
const SECRET_KEY: &str = "CF_ACCESS_CLIENT_SECRET";

#[derive(Debug, Clone)]
pub struct CfAccess {
    client_id: HeaderValue,
    client_secret: HeaderValue,
}

impl CfAccess {
    pub fn new(client_id: &str, client_secret: &str) -> Result<Self, ConfigError> {
        let client_id = header_value(ID_KEY, client_id)?;
        let mut client_secret = header_value(SECRET_KEY, client_secret)?;
        client_secret.set_sensitive(true);
        Ok(Self {
            client_id,
            client_secret,
        })
    }

    pub fn insert_into(&self, headers: &mut HeaderMap) {
        headers.insert(ID_HEADER, self.client_id.clone());
        headers.insert(SECRET_HEADER, self.client_secret.clone());
    }
}

pub fn load_cf_access(
    client_id: Option<String>,
    client_secret: Option<String>,
) -> Result<Option<CfAccess>, ConfigError> {
    match (client_id, client_secret) {
        (None, None) => Ok(None),
        (Some(id), Some(secret)) => CfAccess::new(&id, &secret).map(Some),
        (Some(_), None) => Err(missing(SECRET_KEY)),
        (None, Some(_)) => Err(missing(ID_KEY)),
    }
}

fn header_value(key: &str, raw: &str) -> Result<HeaderValue, ConfigError> {
    HeaderValue::from_str(raw).map_err(|_| ConfigError::InvalidValue {
        key: key.into(),
        value: "not a valid HTTP header value".into(),
    })
}

fn missing(key: &str) -> ConfigError {
    ConfigError::InvalidValue {
        key: key.into(),
        value: "missing".into(),
    }
}
