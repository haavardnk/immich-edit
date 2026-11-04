use std::collections::{BTreeMap, HashSet};

const MAX_ACTIONS: usize = 256;
const MAX_KEYS_PER_ACTION: usize = 16;
const MAX_ACTION_ID_LEN: usize = 64;
const MAX_KEY_SPEC_LEN: usize = 32;
const MODIFIERS: [&str; 3] = ["Mod", "Alt", "Shift"];

pub fn shortcuts(bindings: &BTreeMap<String, Vec<String>>) -> Result<(), String> {
    if bindings.len() > MAX_ACTIONS {
        return Err(format!("shortcuts: at most {MAX_ACTIONS} actions"));
    }
    for (action, keys) in bindings {
        if !action_id(action) {
            return Err("shortcuts: invalid action id".into());
        }
        if keys.len() > MAX_KEYS_PER_ACTION {
            return Err(format!(
                "shortcuts: at most {MAX_KEYS_PER_ACTION} keys for {action:?}"
            ));
        }
        let mut listed = HashSet::with_capacity(keys.len());
        for key in keys {
            if !key_spec(key) {
                return Err(format!("shortcuts: invalid key for {action:?}"));
            }
            if !listed.insert(key.as_str()) {
                return Err(format!("shortcuts: {action:?} lists {key:?} twice"));
            }
        }
    }
    Ok(())
}

fn action_id(id: &str) -> bool {
    let mut bytes = id.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    id.len() <= MAX_ACTION_ID_LEN
        && first.is_ascii_alphabetic()
        && bytes.all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
}

fn key_spec(spec: &str) -> bool {
    if spec.is_empty()
        || spec.len() > MAX_KEY_SPEC_LEN
        || !spec.bytes().all(|b| (b'!'..=b'~').contains(&b))
    {
        return false;
    }
    if spec.len() == 1 {
        return key_name(spec);
    }
    let Some((modifiers, key)) = spec.rsplit_once('+') else {
        return key_name(spec);
    };
    key_name(key) && canonical_modifiers(modifiers)
}

fn key_name(key: &str) -> bool {
    match key.as_bytes() {
        [] => false,
        [single] => !single.is_ascii_uppercase(),
        [first, rest @ ..] => {
            first.is_ascii_uppercase() && rest.iter().all(u8::is_ascii_alphanumeric)
        }
    }
}

fn canonical_modifiers(modifiers: &str) -> bool {
    let mut next = 0;
    for modifier in modifiers.split('+') {
        let Some(position) = MODIFIERS.iter().position(|known| *known == modifier) else {
            return false;
        };
        if position < next {
            return false;
        }
        next = position + 1;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bind(entries: &[(&str, &[&str])]) -> BTreeMap<String, Vec<String>> {
        entries
            .iter()
            .map(|(action, keys)| {
                (
                    action.to_string(),
                    keys.iter().map(|key| key.to_string()).collect(),
                )
            })
            .collect()
    }

    #[test]
    fn action_ids_follow_the_pattern() {
        for id in [
            "help",
            "backToGrid",
            "editor.copy",
            "compare.focusCycle",
            "a-b_c",
            "Z9",
            &"a".repeat(64),
        ] {
            assert!(action_id(id), "{id}");
        }
        for id in [
            "",
            "1help",
            ".help",
            "_x",
            "help me",
            "a/b",
            &"a".repeat(65),
        ] {
            assert!(!action_id(id), "{id}");
        }
    }

    #[test]
    fn canonical_key_specs_are_accepted() {
        for spec in [
            "?",
            "+",
            "a",
            "j",
            "1",
            "[",
            "Shift+/",
            "Shift+=",
            "Mod+z",
            "Mod+Shift+z",
            "Alt+Shift+z",
            "Mod+Alt+Shift+ArrowLeft",
            "Escape",
            "Digit1",
            "F1",
            "PageUp",
            "Tab",
            "Delete",
            "Mod+Digit1",
            &format!("Mod+Alt+Shift+K{}", "x".repeat(17)),
        ] {
            assert!(key_spec(spec), "{spec}");
        }
    }

    #[test]
    fn malformed_key_specs_are_refused() {
        for spec in [
            "",
            " ",
            "Shift+ ",
            "a b",
            "é",
            "\t",
            "Shift++",
            "Mod++",
            "Shift+",
            "a+",
            "+z",
            "Ctrl+z",
            "shift+z",
            "Mod+Mod+z",
            "Mod+Shift+Shift+z",
            &format!("Mod+Alt+Shift+K{}", "x".repeat(18)),
            &"k".repeat(32),
        ] {
            assert!(!key_spec(spec), "{spec}");
        }
    }

    #[test]
    fn non_canonical_key_specs_are_refused() {
        for spec in [
            "Z",
            "Mod+Z",
            "Shift+A",
            "Shift+Mod+z",
            "Alt+Mod+k",
            "Shift+Alt+z",
            "Alt+Mod+Shift+ArrowLeft",
            "escape",
            "arrowLeft",
            "Arrow_Left",
            "Page-Up",
        ] {
            assert!(!key_spec(spec), "{spec}");
        }
    }

    #[test]
    fn bindings_accept_overrides_and_unbound_actions() {
        assert!(shortcuts(&BTreeMap::new()).is_ok());
        assert!(
            shortcuts(&bind(&[
                ("help", &["?", "Shift+/"]),
                ("backToGrid", &["Escape"]),
                ("editor.copy", &[]),
            ]))
            .is_ok()
        );
    }

    #[test]
    fn different_actions_may_share_a_key() {
        assert!(
            shortcuts(&bind(&[
                ("backToGrid", &["Escape"]),
                ("closeDialog", &["Escape"]),
                ("loupe.close", &["Escape", "q"]),
            ]))
            .is_ok()
        );
    }

    #[test]
    fn bindings_are_bounded() {
        let keys: Vec<String> = (0..17).map(|n| format!("F{n}")).collect();
        let keys: Vec<&str> = keys.iter().map(String::as_str).collect();
        assert!(shortcuts(&bind(&[("gridExtend", &keys[..16])])).is_ok());
        assert!(
            shortcuts(&bind(&[("gridExtend", &keys)]))
                .unwrap_err()
                .contains("at most 16 keys")
        );
        let actions = |count: usize| -> BTreeMap<String, Vec<String>> {
            (0..count).map(|n| (format!("action{n}"), vec![])).collect()
        };
        assert!(shortcuts(&actions(256)).is_ok());
        assert!(
            shortcuts(&actions(257))
                .unwrap_err()
                .contains("at most 256 actions")
        );
    }

    #[test]
    fn bad_action_ids_and_keys_are_refused() {
        assert!(shortcuts(&bind(&[("9lives", &[])])).is_err());
        assert!(shortcuts(&bind(&[("help", &["Ctrl+h"])])).is_err());
        assert!(shortcuts(&bind(&[("help", &["Shift+Mod+h"])])).is_err());
        assert!(shortcuts(&bind(&[("help", &[""])])).is_err());
    }

    #[test]
    fn a_key_listed_twice_in_one_action_is_refused_by_name() {
        assert_eq!(
            shortcuts(&bind(&[("loupeNav", &["j", "k", "j"])])).unwrap_err(),
            "shortcuts: \"loupeNav\" lists \"j\" twice"
        );
    }

    #[test]
    fn every_message_names_the_field() {
        for bindings in [
            bind(&[("9", &[])]),
            bind(&[("help", &["Ctrl+h"])]),
            bind(&[("help", &["x", "x"])]),
        ] {
            assert!(shortcuts(&bindings).unwrap_err().starts_with("shortcuts"));
        }
    }
}
