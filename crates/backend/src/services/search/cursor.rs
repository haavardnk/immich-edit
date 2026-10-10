use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Cursor {
    Page { page: u32 },
}

impl Cursor {
    pub fn encode(&self) -> String {
        let json = serde_json::to_vec(self).unwrap_or_default();
        URL_SAFE_NO_PAD.encode(json)
    }

    pub fn decode(token: &str) -> Result<Self, AppError> {
        URL_SAFE_NO_PAD
            .decode(token)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .ok_or_else(|| AppError::BadRequest("invalid search cursor".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cursor_survives_a_round_trip_and_garbage_is_rejected() {
        let cursor = Cursor::Page { page: 3 };
        if Cursor::decode(&cursor.encode()).ok() != Some(cursor) {
            panic!("cursor did not round-trip");
        }
        for token in [
            "",
            "not base64!",
            &URL_SAFE_NO_PAD.encode(b"{\"kind\":\"nope\"}"),
        ] {
            if Cursor::decode(token).is_ok() {
                panic!("{token:?} should be rejected");
            }
        }
    }
}
