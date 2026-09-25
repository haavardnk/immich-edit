use std::collections::HashSet;

use chrono::{DateTime, NaiveDate};

use crate::error::AppError;
use crate::immich::ImmichClient;
use crate::immich::dto::AssetDetail;

pub const DEFAULT_TEMPLATE: &str = "{name}_edit";
const MAX_TEMPLATE_CHARS: usize = 64;
const FALLBACK_STEM: &str = "export";
const UNDATED: &str = "undated";

fn forbidden(c: char) -> bool {
    c.is_control() || matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|')
}

fn invalid(message: String) -> AppError {
    AppError::BadRequest(message)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Token {
    Name,
    Date,
    Seq,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Part {
    Text(String),
    Token(Token),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seq {
    pub position: u32,
    pub total: u32,
}

impl Seq {
    pub const SINGLE: Self = Self {
        position: 1,
        total: 1,
    };
}

pub struct NameContext<'a> {
    pub original: &'a str,
    pub date: Option<NaiveDate>,
    pub seq: Seq,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameTemplate {
    parts: Vec<Part>,
}

impl NameTemplate {
    pub fn parse(raw: Option<&str>) -> Result<Self, AppError> {
        let source = raw
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(DEFAULT_TEMPLATE);
        if source.chars().count() > MAX_TEMPLATE_CHARS {
            return Err(invalid("Filename template is too long".into()));
        }
        if source.chars().any(forbidden) {
            return Err(invalid(
                "Filename template contains a character that is not allowed in file names".into(),
            ));
        }
        let mut parts = Vec::new();
        let mut rest = source;
        while let Some(at) = rest.find(['{', '}']) {
            if at > 0 {
                parts.push(Part::Text(rest[..at].to_string()));
            }
            if rest[at..].starts_with('}') {
                return Err(invalid("Filename template has an unmatched }".into()));
            }
            let close = rest[at..]
                .find('}')
                .ok_or_else(|| invalid("Filename template has an unmatched {".into()))?;
            let token = match &rest[at + 1..at + close] {
                "name" => Token::Name,
                "date" => Token::Date,
                "seq" => Token::Seq,
                other => return Err(invalid(format!("Unknown filename token {{{other}}}"))),
            };
            parts.push(Part::Token(token));
            rest = &rest[at + close + 1..];
        }
        if !rest.is_empty() {
            parts.push(Part::Text(rest.to_string()));
        }
        Ok(Self { parts })
    }

    pub fn render(&self, ctx: &NameContext<'_>) -> String {
        let width = ctx.seq.total.max(1).to_string().len();
        let rendered: String = self
            .parts
            .iter()
            .map(|part| match part {
                Part::Text(text) => text.clone(),
                Part::Token(Token::Name) => clean_stem(ctx.original),
                Part::Token(Token::Date) => ctx
                    .date
                    .map_or_else(|| UNDATED.into(), |d| d.format("%Y-%m-%d").to_string()),
                Part::Token(Token::Seq) => format!("{:0width$}", ctx.seq.position),
            })
            .collect();
        let trimmed = rendered.trim_matches(|c: char| c.is_whitespace() || c == '.');
        if trimmed.is_empty() {
            FALLBACK_STEM.into()
        } else {
            trimmed.to_string()
        }
    }
}

fn clean_stem(original: &str) -> String {
    let stem = original.rsplit_once('.').map_or(original, |(s, _)| s);
    stem.chars()
        .map(|c| if forbidden(c) { '_' } else { c })
        .collect()
}

pub fn capture_date(asset: &AssetDetail) -> Option<NaiveDate> {
    [
        asset.local_date_time.as_deref(),
        asset
            .exif_info
            .as_ref()
            .and_then(|e| e.date_time_original.as_deref()),
        asset.file_created_at.as_deref(),
    ]
    .into_iter()
    .flatten()
    .find_map(|s| DateTime::parse_from_rfc3339(s).ok().map(|d| d.date_naive()))
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

pub fn resolve_filename(stem: &str, extension: &str, existing: &[String]) -> String {
    let lower: HashSet<String> = existing.iter().map(|n| n.to_ascii_lowercase()).collect();
    let mut n: u32 = 1;
    loop {
        let candidate = if n == 1 {
            format!("{stem}.{extension}")
        } else {
            format!("{stem}_{n}.{extension}")
        };
        if !lower.contains(&candidate.to_ascii_lowercase()) {
            return candidate;
        }
        n += 1;
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    const CASES: &str = include_str!("naming_cases.json");

    fn render(case: &Value) -> Result<String, String> {
        let template = NameTemplate::parse(case["template"].as_str()).map_err(|e| match e {
            AppError::BadRequest(message) => message,
            other => format!("{other:?}"),
        })?;
        let date = case["date"]
            .as_str()
            .map(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").unwrap());
        let seq = Seq {
            position: case["position"].as_u64().unwrap_or(1) as u32,
            total: case["total"].as_u64().unwrap_or(1) as u32,
        };
        Ok(template.render(&NameContext {
            original: case["original"].as_str().unwrap_or("IMG_0001.ARW"),
            date,
            seq,
        }))
    }

    #[test]
    fn shared_cases_match() {
        let cases: Vec<Value> = serde_json::from_str(CASES).unwrap();
        for case in &cases {
            let got = render(case);
            match (case["expect"].as_str(), case["error"].as_str()) {
                (Some(want), _) => assert_eq!(got, Ok(want.to_string()), "{case}"),
                (_, Some(want)) => assert_eq!(got, Err(want.to_string()), "{case}"),
                _ => panic!("case without expect or error: {case}"),
            }
        }
    }

    #[test]
    fn capture_date_prefers_local_then_exif_then_file() {
        let asset: AssetDetail = serde_json::from_value(serde_json::json!({
            "id": "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
            "localDateTime": "2025-12-31T23:30:00.000Z",
            "fileCreatedAt": "2026-01-02T00:00:00.000Z",
            "exifInfo": { "dateTimeOriginal": "2026-01-01T00:30:00+01:00" }
        }))
        .unwrap();
        assert_eq!(capture_date(&asset), NaiveDate::from_ymd_opt(2025, 12, 31));
        let exif_only = AssetDetail {
            local_date_time: None,
            ..asset.clone()
        };
        assert_eq!(
            capture_date(&exif_only),
            NaiveDate::from_ymd_opt(2026, 1, 1)
        );
        let file_only = AssetDetail {
            exif_info: None,
            ..exif_only
        };
        assert_eq!(
            capture_date(&file_only),
            NaiveDate::from_ymd_opt(2026, 1, 2)
        );
    }
}
