use rangular_host::{
    max_length, max_length_value, min_length, min_length_value, pattern, pattern_value, required,
    required_value, show_when_dirty, Regex, Value,
};

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

#[test]
fn min_length_skips_empty_and_checks_boundary() {
    assert_eq!(min_length("", 3), None);
    assert_eq!(min_length("ab", 3), Some("Minimum length is not met"));
    assert_eq!(min_length("abc", 3), None);
    assert_eq!(min_length("abcd", 3), None);
    // Unicode scalar values, not bytes.
    assert_eq!(min_length("éé", 3), Some("Minimum length is not met"));
    assert_eq!(min_length("ééé", 3), None);
}

#[test]
fn min_length_value_treats_unit_as_empty() {
    assert_eq!(min_length_value(&Value::Unit, 3), None);
    assert_eq!(
        min_length_value(&Value::Str("ab".into()), 3),
        Some("Minimum length is not met")
    );
    assert_eq!(min_length_value(&Value::Str("abc".into()), 3), None);
}

#[test]
fn max_length_checks_boundary() {
    assert_eq!(max_length("", 2), None);
    assert_eq!(max_length("ab", 2), None);
    assert_eq!(max_length("abc", 2), Some("Maximum length exceeded"));
    assert_eq!(max_length("ééé", 2), Some("Maximum length exceeded"));
}

#[test]
fn max_length_value_treats_unit_as_empty() {
    assert_eq!(max_length_value(&Value::Unit, 2), None);
    assert_eq!(
        max_length_value(&Value::Str("abc".into()), 2),
        Some("Maximum length exceeded")
    );
}

#[test]
fn pattern_match_and_mismatch() {
    let digits = Regex::new(r"^\d+$").expect("digits regex");
    assert_eq!(pattern("123", &digits), None);
    assert_eq!(pattern("12a", &digits), Some("Invalid format"));
    assert_eq!(pattern("", &digits), Some("Invalid format"));
}

#[test]
fn pattern_value_treats_unit_as_empty() {
    let digits = Regex::new(r"^\d+$").expect("digits regex");
    assert_eq!(pattern_value(&Value::Unit, &digits), Some("Invalid format"));
    assert_eq!(pattern_value(&Value::Str("9".into()), &digits), None);
}
