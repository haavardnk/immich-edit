use std::backtrace::{Backtrace, BacktraceStatus};
use std::panic::PanicHookInfo;

pub fn install() {
    std::panic::set_hook(Box::new(log_panic));
}

fn log_panic(info: &PanicHookInfo<'_>) {
    let payload = info.payload();
    let message = payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("non-string panic payload");
    let location = info
        .location()
        .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
        .unwrap_or_default();
    let thread = std::thread::current();
    let thread_name = thread.name().unwrap_or("unnamed");
    let backtrace = Backtrace::capture();
    if backtrace.status() == BacktraceStatus::Captured {
        tracing::error!(target: "panic", panic = message, %location, thread = thread_name, %backtrace, "thread panicked");
        return;
    }
    tracing::error!(target: "panic", panic = message, %location, thread = thread_name, "thread panicked");
}
