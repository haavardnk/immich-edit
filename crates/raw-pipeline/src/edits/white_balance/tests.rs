use super::*;
use crate::color::{KELVIN_MIN, TINT_LIMIT};

const SHOT: Temperature = Temperature {
    kelvin: 5200.0,
    tint: 8.0,
};

#[test]
fn serde_is_tagged_by_mode() {
    let cases = [
        (
            WhiteBalance::AS_SHOT,
            r#"{"mode":"relative","temp":0.0,"tint":0.0}"#,
        ),
        (
            WhiteBalance::Absolute {
                kelvin: 3200.0,
                tint: -6.0,
            },
            r#"{"mode":"absolute","kelvin":3200.0,"tint":-6.0}"#,
        ),
    ];
    for (wb, json) in cases {
        assert_eq!(serde_json::to_string(&wb).unwrap(), json);
        assert_eq!(serde_json::from_str::<WhiteBalance>(json).unwrap(), wb);
    }
    assert!(serde_json::from_str::<WhiteBalance>(r#"{"mode":"absolute"}"#).is_err());
}

#[test]
fn clamping_keeps_each_mode_in_its_range() {
    let cases = [
        (
            WhiteBalance::Relative {
                temp: 400.0,
                tint: -400.0,
            },
            WhiteBalance::Relative {
                temp: 100.0,
                tint: -100.0,
            },
        ),
        (
            WhiteBalance::Absolute {
                kelvin: 900.0,
                tint: 400.0,
            },
            WhiteBalance::Absolute {
                kelvin: KELVIN_MIN,
                tint: TINT_LIMIT,
            },
        ),
    ];
    for (wb, want) in cases {
        assert_eq!(wb.clamped(), want);
    }
}

#[test]
fn shifting_stays_in_the_stored_mode() {
    let relative = WhiteBalance::Relative {
        temp: 90.0,
        tint: 0.0,
    }
    .shifted(30.0, -10.0);
    assert_eq!(
        relative,
        WhiteBalance::Relative {
            temp: 100.0,
            tint: -10.0
        }
    );
    let absolute = WhiteBalance::from(SHOT).shifted(30.0, -10.0);
    assert_eq!(absolute, SHOT.shifted(30.0, -10.0).into());
}

#[test]
fn resolve_reads_relative_against_as_shot() {
    let pinned = WhiteBalance::Absolute {
        kelvin: 3200.0,
        tint: 4.0,
    };
    let warmer = WhiteBalance::Relative {
        temp: 40.0,
        tint: 0.0,
    };
    let unchanged = WhiteBalance::AS_SHOT.resolve(SHOT);
    assert!((unchanged.kelvin - SHOT.kelvin).abs() < 1e-9 && unchanged.tint == SHOT.tint);
    assert_eq!(warmer.resolve(SHOT), SHOT.shifted(40.0, 0.0));
    assert!(warmer.resolve(SHOT).kelvin > SHOT.kelvin);
    assert_eq!(
        pinned.resolve(SHOT),
        Temperature {
            kelvin: 3200.0,
            tint: 4.0
        }
    );
}
