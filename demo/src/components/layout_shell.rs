use leptos::prelude::*;
use rangular_aot::HostCell;
use rangular_host::{Host, HostError, Value};

include!(concat!(env!("OUT_DIR"), "/rangular/layout_shell_view.rs"));

#[component]
pub fn LayoutShellPanel(tick: RwSignal<u32>) -> impl IntoView {
    let title = RwSignal::new(String::from("Layout shell"));

    Effect::new(move |_| {
        let n = tick.get();
        if n == 0 {
            return;
        }
        title.set(format!("Layout shell (seed {n})"));
    });

    layout_shell_view(
        HostCell::new(LayoutShellHost { title }),
        Box::new(|| view! { <p>"Projected panel body via ng-content"</p> }.into_any()),
    )
}

struct LayoutShellHost {
    title: RwSignal<String>,
}

impl Host for LayoutShellHost {
    fn get(&self, name: &str) -> Option<Value> {
        (name == "title").then(|| Value::Str(self.title.get()))
    }

    fn call(&mut self, _: &str, _: &[Value]) -> Result<Value, HostError> {
        Ok(Value::Unit)
    }
}
