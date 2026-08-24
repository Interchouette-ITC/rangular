use leptos::prelude::*;

const CRAB_COUNT: u32 = 7;
const MARK_COUNT: u32 = 6;
const ASSET_ICON_SRC: &str = "/logo-256.png";

#[component]
pub fn DemoDecor(tick: RwSignal<u32>) -> impl IntoView {
    view! {
        <div class="demo-decor" class=("demo-decor--shuffle", move || tick.get() > 0) aria-hidden="true">
            {(0..CRAB_COUNT)
                .map(|index| {
                    view! {
                        <span class="demo-decor__crab" style=move || decor_style(tick.get(), index, 0)>
                            "🦀"
                        </span>
                    }
                })
                .collect_view()}
            <div class="demo-decor__mark-field">
                {(0..MARK_COUNT)
                    .map(|index| {
                        view! {
                            <span
                                class="asset-icon demo-decor__mark"
                                style=move || mark_style(tick.get(), index)
                            >
                                <img class="asset-icon__img" src=ASSET_ICON_SRC alt="" />
                            </span>
                        }
                    })
                    .collect_view()}
            </div>
        </div>
    }
}

fn mark_style(tick: u32, index: u32) -> String {
    if tick == 0 {
        style_from_hash(decor_hash(0, index, 1), 1, true)
    } else {
        style_from_hash(mark_shuffle_hash(tick, index), 1, false)
    }
}

fn decor_style(seed: u32, index: u32, kind: u32) -> String {
    style_from_hash(decor_hash(seed, index, kind), kind, false)
}

const fn decor_hash(seed: u32, index: u32, kind: u32) -> u32 {
    seed.wrapping_mul(0x9E37_79B9)
        .wrapping_add(index.wrapping_mul(0x85EB_CA6B))
        .wrapping_add(kind.wrapping_mul(0xC2B2_AE35))
}

const fn mark_shuffle_hash(tick: u32, index: u32) -> u32 {
    tick.wrapping_mul(0x9E37_79B9)
        .wrapping_add(index.wrapping_mul(0x517C_C911))
        .wrapping_add(0xC2B2_AE35)
}

fn style_from_hash(h: u32, kind: u32, mark_load: bool) -> String {
    let (left, top) = if kind == 0 {
        (
            f64::from(h % 86) + 4.0,
            f64::from((h >> 8) % 80) + 6.0,
        )
    } else if mark_load {
        (
            f64::from((h >> 3) % 86) + 4.0,
            f64::from((h >> 11) % 80) + 6.0,
        )
    } else {
        (
            f64::from((h >> 2) % 86) + 4.0,
            f64::from((h >> 10) % 80) + 6.0,
        )
    };
    let rotate = f64::from((h >> 16) % 80) - 40.0;
    let scale = if kind == 0 {
        0.85 + f64::from((h >> 24) % 30) / 100.0
    } else {
        0.65 + f64::from((h >> 24) % 35) / 100.0
    };
    format!("left: {left:.1}%; top: {top:.1}%; transform: rotate({rotate:.1}deg) scale({scale:.2});")
}
