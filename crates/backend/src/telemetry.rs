use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

mod chain;
pub mod http;
mod panic;

pub use chain::ErrorChain;

const DEFAULT_FILTER: &str = "info";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LogFormat {
    Pretty,
    Json,
}

fn parse_format(raw: Option<&str>) -> anyhow::Result<LogFormat> {
    match raw.map(str::trim).unwrap_or("") {
        "" | "pretty" => Ok(LogFormat::Pretty),
        "json" => Ok(LogFormat::Json),
        other => anyhow::bail!("LOG_FORMAT must be \"pretty\" or \"json\", got {other:?}"),
    }
}

pub fn init() -> anyhow::Result<()> {
    let format = parse_format(std::env::var("LOG_FORMAT").ok().as_deref())?;
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER));
    let registry = tracing_subscriber::registry().with(filter);
    match format {
        LogFormat::Pretty => registry.with(fmt::layer()).init(),
        LogFormat::Json => registry
            .with(
                fmt::layer()
                    .json()
                    .flatten_event(true)
                    .with_current_span(true)
                    .with_span_list(false),
            )
            .init(),
    }
    panic::install();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{LogFormat, parse_format};

    #[test]
    fn parses_log_format() {
        let cases = [
            (None, Some(LogFormat::Pretty)),
            (Some(""), Some(LogFormat::Pretty)),
            (Some("pretty"), Some(LogFormat::Pretty)),
            (Some(" json "), Some(LogFormat::Json)),
            (Some("JSON"), None),
            (Some("logfmt"), None),
        ];
        for (raw, want) in cases {
            let got = parse_format(raw).ok();
            if got != want {
                panic!("{raw:?}: got {got:?}, want {want:?}");
            }
        }
    }
}
