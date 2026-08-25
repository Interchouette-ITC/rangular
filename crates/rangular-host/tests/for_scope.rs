use rangular_host::{for_implicit_value, Value};

#[test]
fn for_implicit_index_and_count() {
    assert_eq!(for_implicit_value("$index", 2, 5), Some(Value::Num(2.0)));
    assert_eq!(for_implicit_value("$count", 2, 5), Some(Value::Num(5.0)));
}

#[test]
fn for_implicit_first_last() {
    assert_eq!(for_implicit_value("$first", 0, 3), Some(Value::Bool(true)));
    assert_eq!(for_implicit_value("$first", 1, 3), Some(Value::Bool(false)));
    assert_eq!(for_implicit_value("$last", 2, 3), Some(Value::Bool(true)));
    assert_eq!(for_implicit_value("$last", 1, 3), Some(Value::Bool(false)));
}

#[test]
fn for_implicit_even_odd() {
    assert_eq!(for_implicit_value("$even", 0, 4), Some(Value::Bool(true)));
    assert_eq!(for_implicit_value("$even", 1, 4), Some(Value::Bool(false)));
    assert_eq!(for_implicit_value("$odd", 1, 4), Some(Value::Bool(true)));
    assert_eq!(for_implicit_value("$odd", 2, 4), Some(Value::Bool(false)));
}

#[test]
fn for_implicit_unknown_is_none() {
    assert_eq!(for_implicit_value("$foo", 0, 1), None);
}
