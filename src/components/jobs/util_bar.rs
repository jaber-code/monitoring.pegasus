use leptos::prelude::*;

/// A compact horizontal utilization meter: `[====      ] 41%`.
///
/// `value` is a percentage (0–100); `None` renders a muted `--` (e.g. a job
/// that has not run yet). `kind` (`gpu` / `cpu` / `mem`) selects the fill
/// colour via CSS.
///
/// Non-reactive by design: the table re-creates every row when its data
/// changes, so there is nothing here to keep live.
#[component]
pub fn UtilBar(value: Option<f32>, kind: &'static str) -> impl IntoView {
    match value {
        None => view! { <span class="util util--empty">"--"</span> }.into_any(),
        Some(v) => {
            let pct = v.clamp(0.0, 100.0);
            let width = format!("width:{pct:.0}%");
            let label = format!("{pct:.0}%");
            let class = format!("util util--{kind}");
            view! {
                <div class=class>
                    <div class="util__track">
                        <div class="util__fill" style=width></div>
                    </div>
                    <span class="util__label">{label}</span>
                </div>
            }
            .into_any()
        }
    }
}
