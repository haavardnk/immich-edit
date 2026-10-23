use std::sync::Arc;

use ort::environment::EnvironmentBuilder;
use ort::logging::LogLevel;
use tracing::Level;

const TARGET: &str = "onnxruntime";

pub(super) fn routed(builder: EnvironmentBuilder) -> EnvironmentBuilder {
    builder.with_logger(Arc::new(|level, _category, _id, location, message| {
        forward(level, location, message)
    }))
}

pub(super) fn threshold() -> LogLevel {
    if tracing::enabled!(target: TARGET, Level::TRACE) {
        LogLevel::Verbose
    } else if tracing::enabled!(target: TARGET, Level::DEBUG) {
        LogLevel::Info
    } else {
        LogLevel::Warning
    }
}

fn forward(level: LogLevel, location: &str, message: &str) {
    match level {
        LogLevel::Verbose => tracing::trace!(target: TARGET, location, "{message}"),
        LogLevel::Info => tracing::debug!(target: TARGET, location, "{message}"),
        LogLevel::Warning => tracing::warn!(target: TARGET, location, "{message}"),
        LogLevel::Error | LogLevel::Fatal => {
            tracing::error!(target: TARGET, location, "{message}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::test_logs::capture;

    #[test]
    fn info_is_debug_detail_and_warnings_stay_visible() {
        let (logs, _guard) = capture(Level::INFO);
        forward(LogLevel::Verbose, "graph.cc:1", "Removing NodeArg");
        forward(LogLevel::Info, "program_manager.cc:1", "Starting program");
        forward(
            LogLevel::Warning,
            "coreml.cc:1",
            "partitions supported by CoreML",
        );
        forward(LogLevel::Fatal, "session.cc:1", "device lost");
        let text = logs.text();
        assert!(!text.contains("Removing NodeArg"), "{text}");
        assert!(!text.contains("Starting program"), "{text}");
        assert!(
            text.contains("WARN onnxruntime: partitions supported by CoreML"),
            "{text}"
        );
        assert!(text.contains("location=\"coreml.cc:1\""), "{text}");
        assert!(text.contains("ERROR onnxruntime: device lost"), "{text}");
    }

    #[test]
    fn the_threshold_follows_the_onnxruntime_target() {
        let cases = [
            (Level::INFO, LogLevel::Warning),
            (Level::DEBUG, LogLevel::Info),
            (Level::TRACE, LogLevel::Verbose),
        ];
        for (max, want) in cases {
            let (_logs, _guard) = capture(max);
            assert_eq!(threshold(), want, "{max}");
        }
    }
}
