const MAX_NAME_CHARS: usize = 80;

pub fn display_name(raw: &str, label: &str) -> Result<String, String> {
    let name = raw.trim();
    if name.is_empty() {
        return Err(format!("{label} required"));
    }
    if name.chars().count() > MAX_NAME_CHARS {
        return Err(format!("{label} longer than {MAX_NAME_CHARS} characters"));
    }
    if name.chars().any(char::is_control) {
        return Err(format!("{label} must not contain control characters"));
    }
    Ok(name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_is_trimmed() {
        assert_eq!(
            display_name("  Warm look \n", "snapshot name").unwrap(),
            "Warm look"
        );
    }

    #[test]
    fn a_blank_name_names_its_label() {
        for blank in ["", "   ", "\t\n"] {
            assert_eq!(
                display_name(blank, "snapshot name").unwrap_err(),
                "snapshot name required"
            );
        }
    }

    #[test]
    fn a_name_counts_characters_not_bytes() {
        assert!(display_name(&"x".repeat(80), "snapshot name").is_ok());
        assert!(display_name(&"é".repeat(80), "snapshot name").is_ok());
        assert!(display_name(&format!("  {}  ", "x".repeat(80)), "snapshot name").is_ok());
        for long in ["x".repeat(81), "é".repeat(81)] {
            assert_eq!(
                display_name(&long, "snapshot name").unwrap_err(),
                "snapshot name longer than 80 characters"
            );
        }
    }

    #[test]
    fn a_name_refuses_inner_control_characters() {
        for name in [
            "Warm\nlook",
            "Warm\tlook",
            "Warm\u{7}",
            "\u{1b}[31mWarm",
            "Warm\u{85}look",
        ] {
            assert_eq!(
                display_name(name, "album name").unwrap_err(),
                "album name must not contain control characters"
            );
        }
        assert!(display_name("Ferien – Nordsee 🌊", "album name").is_ok());
    }
}
