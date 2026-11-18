use std::collections::HashSet;

use crate::services::edits_store::CropRatio;

const MAX_RATIOS: usize = 16;
const MAX_SIDE: i64 = 9999;

pub fn crop_ratios(ratios: &[CropRatio]) -> Result<(), String> {
    if ratios.len() > MAX_RATIOS {
        return Err(format!("crop_ratios: at most {MAX_RATIOS} entries"));
    }
    let mut seen = HashSet::with_capacity(ratios.len());
    for ratio in ratios {
        if !side(ratio.num) || !side(ratio.den) {
            return Err(format!(
                "crop_ratios: sides must be integers from 1 to {MAX_SIDE}"
            ));
        }
        let normalized = ratio.normalized();
        if !seen.insert(normalized) {
            return Err(format!(
                "crop_ratios: duplicate ratio {}:{}",
                normalized.num, normalized.den
            ));
        }
    }
    Ok(())
}

fn side(value: i64) -> bool {
    (1..=MAX_SIDE).contains(&value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ratio(num: i64, den: i64) -> CropRatio {
        CropRatio { num, den }
    }

    #[test]
    fn sides_are_bounded_integers() {
        assert!(crop_ratios(&[]).is_ok());
        assert!(crop_ratios(&[ratio(1, 1), ratio(9999, 1), ratio(2, 9999)]).is_ok());
        for bad in [
            ratio(0, 1),
            ratio(1, 0),
            ratio(-3, 2),
            ratio(10000, 1),
            ratio(3, 10000),
        ] {
            assert!(
                crop_ratios(&[bad])
                    .unwrap_err()
                    .starts_with("crop_ratios: sides"),
                "{bad:?}"
            );
        }
    }

    #[test]
    fn at_most_sixteen_ratios() {
        let full: Vec<CropRatio> = (1..=16).map(|n| ratio(n + 1, 1)).collect();
        assert!(crop_ratios(&full).is_ok());
        let over: Vec<CropRatio> = (1..=17).map(|n| ratio(n + 1, 1)).collect();
        assert_eq!(
            crop_ratios(&over).unwrap_err(),
            "crop_ratios: at most 16 entries"
        );
    }

    #[test]
    fn a_ratio_and_its_rotation_are_duplicates() {
        assert_eq!(
            crop_ratios(&[ratio(3, 2), ratio(2, 3)]).unwrap_err(),
            "crop_ratios: duplicate ratio 3:2"
        );
        assert!(crop_ratios(&[ratio(4, 4), ratio(1, 1)]).is_ok());
    }
}
