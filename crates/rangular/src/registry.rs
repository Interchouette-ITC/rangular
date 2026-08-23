use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

/// Custom element / panel tags for a typical app shell.
pub const APP_ROOT: &str = "app-root";
pub const APP_SITE_HEADER: &str = "app-site-header";
pub const APP_SEED_BAR: &str = "app-seed-bar";
pub const APP_PREVIEW: &str = "app-preview";
pub const APP_ACCESSORIES: &str = "app-accessories";

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

    /// Register the default panel tags for the app shell.
    #[must_use]
    pub fn with_default_panels() -> Self {
        let mut reg = Self::new();
        for (tag, name) in [
            (APP_ROOT, "AppRoot"),
            (APP_SITE_HEADER, "SiteHeader"),
            (APP_SEED_BAR, "SeedBar"),
            (APP_PREVIEW, "PreviewPanel"),
            (APP_ACCESSORIES, "AccessoriesPanel"),
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
    fn default_panels_resolve() {
        let reg = Registry::with_default_panels();
        assert_eq!(
            reg.resolve(APP_SEED_BAR).map(|e| e.name.as_str()),
            Some("SeedBar")
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
