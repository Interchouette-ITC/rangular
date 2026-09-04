use leptos::prelude::*;
use rangular_aot::HostCell;
use rangular_host::{
    first_error, min_length, pattern, required, Host, HostError, Regex, Value,
};

include!(concat!(env!("OUT_DIR"), "/rangular/field_validators_view.rs"));

#[component]
pub fn FieldValidatorsPanel(applied_seed: RwSignal<String>) -> impl IntoView {
    let name = RwSignal::new(String::new());
    let name_dirty = RwSignal::new(false);
    let code = RwSignal::new(String::new());
    let code_dirty = RwSignal::new(false);
    let code_re = Regex::new(r"^\d{4}$").expect("four-digit code regex");

    Effect::new(move |_| {
        let seed = applied_seed.get();
        name_dirty.set(false);
        code_dirty.set(false);
        if seed.is_empty() {
            name.set(String::new());
            code.set(String::new());
            return;
        }
        let n = crate::demo_seed::seed_to_tick(&seed);
        if n.is_multiple_of(3) {
            name.set(String::new());
            code.set(String::new());
        } else {
            name.set(format!("User {seed}"));
            code.set(format!("{:04}", n % 10_000));
        }
    });

    field_validators_view(HostCell::new(FieldValidatorsHost {
        name,
        name_dirty,
        code,
        code_dirty,
        code_re,
    }))
}

struct FieldValidatorsHost {
    name: RwSignal<String>,
    name_dirty: RwSignal<bool>,
    code: RwSignal<String>,
    code_dirty: RwSignal<bool>,
    code_re: Regex,
}

impl FieldValidatorsHost {
    fn name_error(&self) -> Option<&'static str> {
        let value = self.name.get();
        first_error(&[required(&value), min_length(&value, 3)])
    }

    fn code_error(&self) -> Option<&'static str> {
        let value = self.code.get();
        first_error(&[required(&value), pattern(&value, &self.code_re)])
    }
}

impl Host for FieldValidatorsHost {
    fn get(&self, key: &str) -> Option<Value> {
        match key {
            "name" => Some(Value::Str(self.name.get())),
            "nameDirty" => Some(Value::Bool(self.name_dirty.get())),
            "nameInvalid" => Some(Value::Bool(self.name_error().is_some())),
            "nameError" => Some(Value::Str(self.name_error().unwrap_or("").to_owned())),
            "code" => Some(Value::Str(self.code.get())),
            "codeDirty" => Some(Value::Bool(self.code_dirty.get())),
            "codeInvalid" => Some(Value::Bool(self.code_error().is_some())),
            "codeError" => Some(Value::Str(self.code_error().unwrap_or("").to_owned())),
            _ => None,
        }
    }

    fn set(&mut self, key: &str, value: Value) -> Result<(), HostError> {
        match key {
            "name" => {
                self.name_dirty.set(true);
                if let Some(s) = value.as_str() {
                    self.name.set(s.to_owned());
                }
            }
            "code" => {
                self.code_dirty.set(true);
                if let Some(s) = value.as_str() {
                    self.code.set(s.to_owned());
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn call(&mut self, _: &str, _: &[Value]) -> Result<Value, HostError> {
        Ok(Value::Unit)
    }
}
