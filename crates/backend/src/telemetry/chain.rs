use std::error::Error;
use std::fmt::{self, Write};

pub struct ErrorChain<'a>(pub &'a (dyn Error + 'static));

impl fmt::Display for ErrorChain<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut text = self.0.to_string();
        let mut source = self.0.source();
        while let Some(err) = source {
            let part = err.to_string();
            if !text.contains(&part) {
                text.push_str(": ");
                text.push_str(&part);
            }
            source = err.source();
        }
        for c in text.chars() {
            if c.is_control() {
                write!(f, "{}", c.escape_default())?;
            } else {
                f.write_char(c)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ErrorChain;

    #[derive(Debug, thiserror::Error)]
    #[error("{text}")]
    struct Link {
        text: &'static str,
        #[source]
        source: Option<Box<Link>>,
    }

    fn chain(parts: &[&'static str]) -> Link {
        let mut link: Option<Box<Link>> = None;
        for text in parts.iter().rev() {
            link = Some(Box::new(Link { text, source: link }));
        }
        *link.expect("chain needs a link")
    }

    #[test]
    fn joins_sources_and_skips_repeats() {
        let cases: [(&[&str], &str); 5] = [
            (&["boom"], "boom"),
            (
                &["send request", "connect", "connection refused"],
                "send request: connect: connection refused",
            ),
            (
                &["io error: permission denied", "permission denied"],
                "io error: permission denied",
            ),
            (&["outer", "", "inner"], "outer: inner"),
            (
                &["bad\nline", "esc\u{1b}[31m"],
                "bad\\nline: esc\\u{1b}[31m",
            ),
        ];
        for (parts, want) in cases {
            let err = chain(parts);
            let got = ErrorChain(&err).to_string();
            if got != want {
                panic!("{parts:?}: got {got:?}, want {want:?}");
            }
        }
    }
}
