mod common;

use immich_edit_backend::routes::export::resolve_filename;

#[test]
fn resolves_with_no_existing() {
    let name = resolve_filename("DSC0001.ARW", "_edit", "jpg", &["DSC0001.ARW".into()]);
    assert_eq!(name, "DSC0001_edit.jpg");
}

#[test]
fn resolves_increments_on_collision() {
    let existing = vec!["DSC0001.ARW".into(), "DSC0001_edit.jpg".into()];
    let name = resolve_filename("DSC0001.ARW", "_edit", "jpg", &existing);
    assert_eq!(name, "DSC0001_edit_2.jpg");
}

#[test]
fn resolves_skips_multiple_collisions() {
    let existing = vec![
        "DSC0001.ARW".into(),
        "DSC0001_edit.jpg".into(),
        "DSC0001_edit_2.jpg".into(),
        "DSC0001_edit_3.jpg".into(),
    ];
    let name = resolve_filename("DSC0001.ARW", "_edit", "jpg", &existing);
    assert_eq!(name, "DSC0001_edit_4.jpg");
}

#[test]
fn resolves_case_insensitive() {
    let existing = vec!["IMG.JPG".into(), "IMG_EDIT.JPG".into()];
    let name = resolve_filename("IMG.JPG", "_edit", "jpg", &existing);
    assert_eq!(name, "IMG_edit_2.jpg");
}

#[test]
fn resolves_handles_no_extension_original() {
    let name = resolve_filename("raw", "_edit", "png", &["raw".into()]);
    assert_eq!(name, "raw_edit.png");
}

#[test]
fn resolves_custom_suffix() {
    let name = resolve_filename("a.arw", "-final", "tif", &["a.arw".into()]);
    assert_eq!(name, "a-final.tif");
}
