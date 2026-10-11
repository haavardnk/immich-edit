use super::*;

#[test]
fn standard_illuminants_read_like_lightroom() {
    for (xy, kelvin, tint) in [
        ([0.44757, 0.40745], 2856.0, 0.0),
        ([0.34567, 0.35850], 5002.0, 9.6),
        ([0.31271, 0.32902], 6503.0, 9.8),
    ] {
        let t = Temperature::from_xy(xy);
        assert!((t.kelvin - kelvin).abs() < 1.0, "{xy:?}: {t:?}");
        assert!((t.tint - tint).abs() < 0.1, "{xy:?}: {t:?}");
    }
}

#[test]
fn xy_round_trips_across_the_range() {
    for kelvin in [2000.0, 2856.0, 4000.0, 6504.0, 10000.0, 30000.0, 50000.0] {
        for tint in [-150.0, -40.0, 0.0, 40.0, 150.0] {
            let back = Temperature::from_xy(Temperature { kelvin, tint }.xy());
            assert!(
                (back.kelvin - kelvin).abs() / kelvin < 1e-5,
                "{kelvin} {tint}: {back:?}"
            );
            assert!((back.tint - tint).abs() < 1e-3, "{kelvin} {tint}: {back:?}");
        }
    }
}

#[test]
fn positive_tint_sits_above_the_locus() {
    let on = Temperature {
        kelvin: 5000.0,
        tint: 0.0,
    }
    .xy();
    let green = Temperature {
        kelvin: 5000.0,
        tint: 50.0,
    }
    .xy();
    assert!(green[1] > on[1], "{green:?} vs {on:?}");
}

#[test]
fn relative_shift_scales_mired_and_clamps() {
    let shot = Temperature {
        kelvin: 5000.0,
        tint: 10.0,
    };
    let warm = shot.shifted(100.0, 20.0);
    assert!(
        (warm.kelvin - 5000.0 * 2f64.powf(1.5)).abs() < 1e-6,
        "{warm:?}"
    );
    assert!((warm.tint - 28.0).abs() < 1e-9, "{warm:?}");
    let floor = shot.shifted(-300.0, -300.0);
    assert_eq!(floor.kelvin, KELVIN_MIN);
    assert_eq!(floor.tint, -TINT_LIMIT);
}
