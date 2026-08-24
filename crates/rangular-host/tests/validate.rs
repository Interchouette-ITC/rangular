use rangular_host::{required, required_value, Value};

#[test]
fn required_rejects_empty_and_whitespace() {
    assert_eq!(required(""), Some("This field is required"));
    assert_eq!(required("   "), Some("This field is required"));
    assert_eq!(required("ok"), None);
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
