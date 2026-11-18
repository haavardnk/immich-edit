use serde::Serialize;

use crate::immich::dto::AssetSummary;
use crate::services::search::query::Direction;

#[derive(Debug)]
pub struct Side {
    items: Vec<AssetSummary>,
    more: bool,
}

impl Side {
    pub fn cut(mut nearest_first: Vec<AssetSummary>, wanted: usize) -> Self {
        let more = nearest_first.len() > wanted;
        nearest_first.truncate(wanted);
        Self {
            items: nearest_first,
            more,
        }
    }
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Window {
    pub items: Vec<AssetSummary>,
    pub has_before: bool,
    pub has_after: bool,
}

impl Window {
    pub fn alone(anchor: AssetSummary) -> Self {
        Self {
            items: vec![anchor],
            has_before: false,
            has_after: false,
        }
    }

    pub fn around(before: Side, anchor: AssetSummary, after: Side) -> Self {
        Self {
            items: before
                .items
                .into_iter()
                .rev()
                .chain(std::iter::once(anchor))
                .chain(after.items)
                .collect(),
            has_before: before.more,
            has_after: after.more,
        }
    }

    pub fn by_key(direction: Direction, greater: Side, anchor: AssetSummary, lesser: Side) -> Self {
        match direction {
            Direction::Desc => Self::around(greater, anchor, lesser),
            Direction::Asc => Self::around(lesser, anchor, greater),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use uuid::Uuid;

    use super::*;

    fn assets(count: usize) -> Vec<AssetSummary> {
        (0..count)
            .map(|_| serde_json::from_value(json!({ "id": Uuid::new_v4() })).unwrap())
            .collect()
    }

    fn ids(assets: &[AssetSummary]) -> Vec<Uuid> {
        assets.iter().map(|asset| asset.id.source()).collect()
    }

    #[test]
    fn a_side_keeps_what_was_wanted_and_flags_the_lookahead() {
        for (found, wanted, kept, more) in [
            (0, 0, 0, false),
            (1, 0, 0, true),
            (3, 3, 3, false),
            (4, 3, 3, true),
            (2, 5, 2, false),
        ] {
            let side = Side::cut(assets(found), wanted);
            if side.items.len() != kept || side.more != more {
                panic!("{found} found for {wanted} wanted gave {side:?}");
            }
        }
    }

    #[test]
    fn sides_are_laid_out_in_sort_order_around_the_anchor() {
        let greater = assets(3);
        let lesser = assets(2);
        let anchor = assets(1);
        let descending = Window::by_key(
            Direction::Desc,
            Side::cut(greater.clone(), 2),
            anchor[0].clone(),
            Side::cut(lesser.clone(), 2),
        );
        let expected = [
            greater[1].id,
            greater[0].id,
            anchor[0].id,
            lesser[0].id,
            lesser[1].id,
        ]
        .map(|key| key.source());
        if ids(&descending.items) != expected || !descending.has_before || descending.has_after {
            panic!("descending window was {descending:?}");
        }
        let ascending = Window::by_key(
            Direction::Asc,
            Side::cut(greater.clone(), 3),
            anchor[0].clone(),
            Side::cut(lesser.clone(), 1),
        );
        let expected = [
            lesser[0].id,
            anchor[0].id,
            greater[0].id,
            greater[1].id,
            greater[2].id,
        ]
        .map(|key| key.source());
        if ids(&ascending.items) != expected || !ascending.has_before || ascending.has_after {
            panic!("ascending window was {ascending:?}");
        }
    }
}
