const MAX_NAME_CHARS: usize = 80;
const MAX_DESCRIPTION_CHARS: usize = 2000;

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

pub fn album_description(raw: &str) -> Result<Option<String>, String> {
    let description = raw.trim();
    if description.is_empty() {
        return Ok(None);
    }
    if description.chars().count() > MAX_DESCRIPTION_CHARS {
        return Err(format!(
            "description longer than {MAX_DESCRIPTION_CHARS} characters"
        ));
    }
    if description
        .chars()
        .any(|c| c.is_control() && c != '\n' && c != '\t')
    {
        return Err("description must not contain control characters".into());
    }
    Ok(Some(description.to_string()))
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

    #[test]
    fn a_blank_description_clears_it() {
        for blank in ["", "  ", "\n\t"] {
            assert_eq!(album_description(blank).unwrap(), None);
        }
    }

    #[test]
    fn a_description_keeps_line_breaks_and_tabs() {
        assert_eq!(
            album_description(" Day one\n\tbeach ").unwrap().as_deref(),
            Some("Day one\n\tbeach")
        );
    }

    #[test]
    fn a_description_counts_characters_up_to_2000() {
        assert!(album_description(&"é".repeat(2000)).is_ok());
        assert_eq!(
            album_description(&"é".repeat(2001)).unwrap_err(),
            "description longer than 2000 characters"
        );
    }

    #[test]
    fn a_description_refuses_other_control_characters() {
        for description in ["Day\rone", "Day\u{7}", "\u{1b}[31mDay"] {
            assert_eq!(
                album_description(description).unwrap_err(),
                "description must not contain control characters"
            );
        }
    }
}
