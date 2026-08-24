use rangular_host::{required, required_value, show_when_dirty, Value};

#[test]
fn required_rejects_empty_and_whitespace() {
    assert_eq!(required(""), Some("This field is required"));
    assert_eq!(required("   "), Some("This field is required"));
    assert_eq!(required("ok"), None);
}

#[test]
fn show_when_dirty_matches_angular_pristine() {
    assert!(!show_when_dirty(true, false));
    assert!(show_when_dirty(true, true));
    assert!(!show_when_dirty(false, true));
}

#[test]
fn required_value_uses_str() {
    assert_eq!(
        required_value(&Value::Str(String::new())),
        Some("This field is required")
    );
    assert_eq!(required_value(&Value::Str("x".into())), None);
    assert_eq!(required_value(&Value::Unit), Some("This field is required"));
}
