use std::collections::HashSet;

use crate::error::AppError;
use crate::immich::ImmichClient;
use crate::immich::dto::AssetDetail;

pub fn validate_suffix(raw: &str) -> Result<String, AppError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok("_edit".into());
    }
    if trimmed
        .chars()
        .any(|c| c.is_control() || matches!(c, '/' | '\\' | '\0'))
    {
        return Err(AppError::BadRequest("invalid filename suffix".into()));
    }
    if trimmed.len() > 32 {
        return Err(AppError::BadRequest("filename suffix too long".into()));
    }
    Ok(trimmed.to_string())
}

pub async fn collect_existing_filenames(
    immich: &ImmichClient,
    original: &AssetDetail,
) -> Vec<String> {
    let mut names = vec![original.original_file_name.clone()];
    let Some(stack_id) = original.stack_id.or(original.stack.as_ref().map(|s| s.id)) else {
        return names;
    };
    match immich.get_stack(stack_id).await {
        Ok(stack) => {
            for asset in stack.assets {
                names.push(asset.original_file_name);
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "fetch stack for filename collision");
        }
    }
    names
}

pub fn resolve_filename(
    original: &str,
    suffix: &str,
    extension: &str,
    existing: &[String],
) -> String {
    let stem = match original.rsplit_once('.') {
        Some((s, _)) => s,
        None => original,
    };
    let lower: HashSet<String> = existing.iter().map(|n| n.to_ascii_lowercase()).collect();
    let mut n: u32 = 1;
    loop {
        let candidate = if n == 1 {
            format!("{stem}{suffix}.{extension}")
        } else {
            format!("{stem}{suffix}_{n}.{extension}")
        };
        if !lower.contains(&candidate.to_ascii_lowercase()) {
            return candidate;
        }
        n += 1;
    }
}
