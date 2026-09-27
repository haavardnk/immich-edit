use url::Url;

use crate::error::AppError;
use crate::services::instance_store::InstanceStore;

pub async fn resolve_immich_base(instance: &InstanceStore) -> Result<Url, AppError> {
    let cfg = instance.get().await?;
    let url = cfg.immich_url.ok_or(AppError::SetupRequired)?;
    Url::parse(&url).map_err(|_| AppError::Internal)
}

pub fn validate_candidate_url(raw: &str) -> Result<Url, AppError> {
    let url =
        Url::parse(raw.trim()).map_err(|_| AppError::BadRequest("invalid immich url".into()))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(AppError::BadRequest("immich url must be http(s)".into()));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(AppError::BadRequest(
            "immich url must not contain credentials".into(),
        ));
    }
    let host = url
        .host()
        .ok_or_else(|| AppError::BadRequest("immich url must include a host".into()))?;
    let blocked = match host {
        url::Host::Ipv4(ip) => {
            ip.is_link_local() || ip.is_multicast() || ip.is_unspecified() || ip.is_broadcast()
        }
        url::Host::Ipv6(ip) => {
            ip.is_multicast() || ip.is_unspecified() || (ip.segments()[0] & 0xffc0) == 0xfe80
        }
        url::Host::Domain(_) => false,
    };
    if blocked {
        return Err(AppError::BadRequest(
            "immich url host is not allowed".into(),
        ));
    }
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::validate_candidate_url;

    #[test]
    fn allows_loopback_and_private_hosts() {
        assert!(validate_candidate_url("http://127.0.0.1:2283").is_ok());
        assert!(validate_candidate_url("http://192.168.1.10:2283").is_ok());
        assert!(validate_candidate_url("https://immich.example.com").is_ok());
    }

    #[test]
    fn blocks_cloud_metadata_and_bad_schemes() {
        assert!(validate_candidate_url("http://169.254.169.254/").is_err());
        assert!(validate_candidate_url("http://0.0.0.0/").is_err());
        assert!(validate_candidate_url("ftp://immich.example.com").is_err());
        assert!(validate_candidate_url("http://user:pass@immich.example.com").is_err());
        assert!(validate_candidate_url("not a url").is_err());
    }
}
