use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

/// Example component tags aligned with `tests/fixtures/components/`.
pub const APP_COLOR_FIELD: &str = "app-color-field";
pub const APP_CHROME_HEADER: &str = "app-chrome-header";
pub const APP_ASSET_ICON: &str = "app-asset-icon";
pub const APP_ITEM_LIST: &str = "app-item-list";

/// Metadata for a registered component tag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentEntry {
    pub tag: String,
    pub name: String,
}

/// Tag + typed service registry (maps to Leptos provide/inject at the app edge).
#[derive(Clone, Default)]
pub struct Registry {
    tags: HashMap<String, ComponentEntry>,
    services: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
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
        for (tag, name) in [
            (APP_COLOR_FIELD, "ColorField"),
            (APP_CHROME_HEADER, "ChromeHeader"),
            (APP_ASSET_ICON, "AssetIcon"),
            (APP_ITEM_LIST, "ItemList"),
        ] {
            reg.register_tag(tag, name);
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
            },
        );
    }

    #[must_use]
    pub fn resolve(&self, tag: &str) -> Option<&ComponentEntry> {
        self.tags.get(tag)
    }

    pub fn tags(&self) -> impl Iterator<Item = &ComponentEntry> {
        self.tags.values()
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
    fn provide_inject_roundtrip() {
        let mut reg = Registry::new();
        reg.provide(42_u32);
        assert_eq!(reg.inject::<u32>(), Some(&42));
        assert!(reg.inject::<i32>().is_none());
    }
}
