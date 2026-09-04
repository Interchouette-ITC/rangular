use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use rangular_expr::{PipeFn, PipeRegistry};
use rangular_parser::{builtin_tag_io, TagIo};

/// Example component tags aligned with `tests/fixtures/components/`.
pub const APP_COLOR_FIELD: &str = "app-color-field";
pub const APP_CHROME_HEADER: &str = "app-chrome-header";
pub const APP_ASSET_ICON: &str = "app-asset-icon";
pub const APP_ITEM_LIST: &str = "app-item-list";
pub const APP_IO_CHILD: &str = "app-io-child";

/// Metadata for a registered component tag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentEntry {
    pub tag: String,
    pub name: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

/// Tag + typed service registry (maps to Leptos provide/inject at the app edge).
#[derive(Clone)]
pub struct Registry {
    tags: HashMap<String, ComponentEntry>,
    services: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
    pipes: PipeRegistry,
}

impl Default for Registry {
    fn default() -> Self {
        Self {
            tags: HashMap::new(),
            services: HashMap::new(),
            pipes: PipeRegistry::with_builtins(),
        }
    }
}

impl Registry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register example component tags that match the fixture corpus.
    #[must_use]
    pub fn with_example_panels() -> Self {
        let mut reg = Self::new();
        let builtin = builtin_tag_io();
        for (tag, name) in [
            (APP_COLOR_FIELD, "ColorField"),
            (APP_CHROME_HEADER, "ChromeHeader"),
            (APP_ASSET_ICON, "AssetIcon"),
            (APP_ITEM_LIST, "ItemList"),
            (APP_IO_CHILD, "IoChild"),
        ] {
            if let Some(io) = builtin.get(tag) {
                reg.register_component(tag, name, io);
            } else {
                reg.register_tag(tag, name);
            }
        }
        reg
    }

    pub fn register_tag(&mut self, tag: impl Into<String>, name: impl Into<String>) {
        let tag = tag.into();
        self.tags.insert(
            tag.clone(),
            ComponentEntry {
                tag,
                name: name.into(),
                inputs: Vec::new(),
                outputs: Vec::new(),
            },
        );
    }

    pub fn register_component(
        &mut self,
        tag: impl Into<String>,
        name: impl Into<String>,
        io: &TagIo,
    ) {
        let tag = tag.into();
        self.tags.insert(
            tag.clone(),
            ComponentEntry {
                tag,
                name: name.into(),
                inputs: io.inputs.clone(),
                outputs: io.outputs.clone(),
            },
        );
    }

    /// Register or replace a custom pipe (same map used by AOT / runtime eval).
    pub fn register_pipe(&mut self, name: impl Into<String>, pipe: PipeFn) {
        self.pipes.register(name, pipe);
    }

    /// Pipe map for [`rangular_expr::eval_with_pipes`] / `HostCell`.
    #[must_use]
    pub const fn pipes(&self) -> &PipeRegistry {
        &self.pipes
    }

    /// Shared pipe registry handle.
    #[must_use]
    pub fn pipes_arc(&self) -> Arc<PipeRegistry> {
        Arc::new(self.pipes.clone())
    }

    #[must_use]
    pub fn resolve(&self, tag: &str) -> Option<&ComponentEntry> {
        self.tags.get(tag)
    }

    pub fn tags(&self) -> impl Iterator<Item = &ComponentEntry> {
        self.tags.values()
    }

    /// Tag → I/O map for [`rangular_parser::classify_bindings`].
    #[must_use]
    pub fn tag_io_map(&self) -> HashMap<String, TagIo> {
        self.tags
            .iter()
            .map(|(tag, entry)| {
                (
                    tag.clone(),
                    TagIo {
                        inputs: entry.inputs.clone(),
                        outputs: entry.outputs.clone(),
                    },
                )
            })
            .collect()
    }

    /// Provide a typed service (replaces any previous value of the same type).
    pub fn provide<T: Send + Sync + 'static>(&mut self, value: T) {
        self.services.insert(TypeId::of::<T>(), Arc::new(value));
    }

    /// Inject a typed service previously provided.
    #[must_use]
    pub fn inject<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.services
            .get(&TypeId::of::<T>())
            .and_then(|v| v.downcast_ref::<T>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rangular_expr::{eval_with_pipes, parse, Host, Value};
    use rangular_host::HostError;

    struct LabelHost;

    impl Host for LabelHost {
        fn get(&self, name: &str) -> Option<Value> {
            (name == "label").then(|| Value::Str("Hi".into()))
        }

        fn call(&mut self, _: &str, _: &[Value]) -> Result<Value, HostError> {
            Ok(Value::Unit)
        }
    }

    #[test]
    fn example_panels_resolve() {
        let reg = Registry::with_example_panels();
        assert_eq!(
            reg.resolve(APP_COLOR_FIELD).map(|e| e.name.as_str()),
            Some("ColorField")
        );
        assert!(reg.resolve("nope").is_none());
    }

    #[test]
    fn io_child_has_inputs_and_outputs() {
        let reg = Registry::with_example_panels();
        let entry = reg.resolve(APP_IO_CHILD).expect("app-io-child");
        assert_eq!(entry.inputs, vec!["label", "muted"]);
        assert_eq!(entry.outputs, vec!["muteToggle"]);
    }

    #[test]
    fn provide_inject_roundtrip() {
        let mut reg = Registry::new();
        reg.provide(42_u32);
        assert_eq!(reg.inject::<u32>(), Some(&42));
        assert!(reg.inject::<i32>().is_none());
    }

    #[test]
    fn custom_pipe_via_registry() {
        let mut reg = Registry::new();
        reg.register_pipe("shout", |v, _| {
            Ok(Value::Str(format!(
                "{}!",
                v.as_str().unwrap_or_default().to_uppercase()
            )))
        });
        let expr = parse("label | shout").expr.expect("pipe expr");
        let mut host = LabelHost;
        assert_eq!(
            eval_with_pipes(&expr, &mut host, reg.pipes()).unwrap(),
            Value::Str("HI!".into())
        );
    }

    #[test]
    fn pipes_arc_tags_and_tag_io_map() {
        let mut reg = Registry::with_example_panels();
        reg.register_tag("app-extra", "Extra");
        let arc = reg.pipes_arc();
        assert!(arc.contains("uppercase"));
        let tags: Vec<_> = reg.tags().map(|e| e.tag.clone()).collect();
        assert!(tags.contains(&APP_COLOR_FIELD.to_owned()));
        assert!(tags.contains(&"app-extra".to_owned()));
        let map = reg.tag_io_map();
        let io = map.get(APP_IO_CHILD).expect("io child");
        assert_eq!(io.inputs, vec!["label", "muted"]);
        assert_eq!(io.outputs, vec!["muteToggle"]);
        let bare = map.get("app-extra").expect("extra");
        assert_eq!(bare.inputs.len(), 0);
    }
}
