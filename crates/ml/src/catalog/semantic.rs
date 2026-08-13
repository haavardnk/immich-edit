pub struct SemanticClass {
    pub id: &'static str,
    pub name: &'static str,
    pub channel: usize,
}

pub const SEMANTIC_CLASSES: &[SemanticClass] = &[
    SemanticClass {
        id: "sky",
        name: "Sky",
        channel: 2,
    },
    SemanticClass {
        id: "water",
        name: "Water",
        channel: 21,
    },
    SemanticClass {
        id: "sea",
        name: "Sea",
        channel: 26,
    },
    SemanticClass {
        id: "tree",
        name: "Trees",
        channel: 4,
    },
    SemanticClass {
        id: "grass",
        name: "Grass",
        channel: 9,
    },
    SemanticClass {
        id: "plant",
        name: "Plants",
        channel: 17,
    },
    SemanticClass {
        id: "mountain",
        name: "Mountain",
        channel: 16,
    },
    SemanticClass {
        id: "rock",
        name: "Rock",
        channel: 34,
    },
    SemanticClass {
        id: "sand",
        name: "Sand",
        channel: 46,
    },
    SemanticClass {
        id: "earth",
        name: "Ground",
        channel: 13,
    },
    SemanticClass {
        id: "building",
        name: "Buildings",
        channel: 1,
    },
    SemanticClass {
        id: "road",
        name: "Road",
        channel: 6,
    },
    SemanticClass {
        id: "person",
        name: "People",
        channel: 12,
    },
];

pub fn semantic_class(id: &str) -> Option<&'static SemanticClass> {
    SEMANTIC_CLASSES.iter().find(|c| c.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_classes_are_unique_and_resolvable() {
        let mut ids: Vec<&str> = SEMANTIC_CLASSES.iter().map(|c| c.id).collect();
        let count = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), count, "class ids must be unique");
        for class in SEMANTIC_CLASSES {
            assert!(class.channel < 150, "{} is outside ADE20K", class.id);
            assert_eq!(
                semantic_class(class.id).map(|c| c.channel),
                Some(class.channel)
            );
        }
        assert_eq!(semantic_class("sky").unwrap().channel, 2);
        assert!(semantic_class("nope").is_none());
    }
}
