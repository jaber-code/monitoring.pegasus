use leptos::prelude::*;

use crate::models::{JobQuery, JobState};

/// Filter controls for the Current Jobs screen.
///
/// The bar is *stateless*: it renders from the current [`JobQuery`] (which the
/// page derives from the URL) and reports every edit through `set`. The page
/// owns the "write it back to the URL" step, so filters stay shareable and
/// survive a reload.
#[component]
pub fn JobsFilterBar(
    #[prop(into)] query: Signal<JobQuery>,
    set: Callback<JobQuery>,
) -> impl IntoView {
    let toggle_state = move |st: JobState| {
        let mut q = query.get_untracked();
        let name = st.as_str().to_string();
        match q.states.iter().position(|s| *s == name) {
            Some(i) => {
                q.states.remove(i);
            }
            None => q.states.push(name),
        }
        set.run(q);
    };

    let chips = JobState::filter_choices()
        .iter()
        .map(|st| {
            let st = *st;
            let active = move || query.get().has_state(st);
            view! {
                <button
                    class="chip chip--toggle"
                    class:is-active=active
                    on:click=move |_| toggle_state(st)
                >
                    {st.label()}
                </button>
            }
        })
        .collect_view();

    let window_value = move || {
        query
            .get()
            .window_hours
            .map(|h| h.to_string())
            .unwrap_or_default()
    };

    let dirty = move || query.get() != JobQuery::default();

    view! {
        <div class="jobfilter">
            <div class="jobfilter__states">
                <span class="jf__label">"State"</span>
                {chips}
            </div>

            <div class="jobfilter__fields">
                <FilterText
                    label="User"
                    placeholder="username"
                    value=Signal::derive(move || query.get().user.clone().unwrap_or_default())
                    on_commit=Callback::new(move |v: String| {
                        let mut q = query.get_untracked();
                        q.user = non_empty(&v);
                        set.run(q);
                    })
                />
                <FilterText
                    label="Job ID"
                    placeholder="e.g. 3341149"
                    value=Signal::derive(move || {
                        query.get().job_id.map(|v| v.to_string()).unwrap_or_default()
                    })
                    on_commit=Callback::new(move |v: String| {
                        let mut q = query.get_untracked();
                        q.job_id = v.trim().parse().ok();
                        set.run(q);
                    })
                />
                <FilterText
                    label="Name"
                    placeholder="job name"
                    value=Signal::derive(move || query.get().name.clone().unwrap_or_default())
                    on_commit=Callback::new(move |v: String| {
                        let mut q = query.get_untracked();
                        q.name = non_empty(&v);
                        set.run(q);
                    })
                />
                <FilterText
                    label="Partition"
                    placeholder="partition"
                    value=Signal::derive(move || query.get().partition.clone().unwrap_or_default())
                    on_commit=Callback::new(move |v: String| {
                        let mut q = query.get_untracked();
                        q.partition = non_empty(&v);
                        set.run(q);
                    })
                />
                <FilterText
                    label="Node"
                    placeholder="node"
                    value=Signal::derive(move || query.get().node.clone().unwrap_or_default())
                    on_commit=Callback::new(move |v: String| {
                        let mut q = query.get_untracked();
                        q.node = non_empty(&v);
                        set.run(q);
                    })
                />

                <label class="jf__field">
                    <span class="jf__label">"Window"</span>
                    <select
                        class="jf__input"
                        prop:value=window_value
                        on:change=move |ev| {
                            let mut q = query.get_untracked();
                            q.window_hours = event_target_value(&ev).parse().ok();
                            set.run(q);
                        }
                    >
                        <option value="">"default"</option>
                        <option value="1">"1h"</option>
                        <option value="6">"6h"</option>
                        <option value="24">"24h"</option>
                        <option value="72">"3d"</option>
                        <option value="168">"7d"</option>
                    </select>
                </label>

                <Show when=dirty>
                    <button
                        class="jf__reset"
                        on:click=move |_| set.run(JobQuery::default())
                    >
                        "Reset"
                    </button>
                </Show>
            </div>
        </div>
    }
}

#[component]
fn FilterText(
    label: &'static str,
    placeholder: &'static str,
    #[prop(into)] value: Signal<String>,
    on_commit: Callback<String>,
) -> impl IntoView {
    view! {
        <label class="jf__field">
            <span class="jf__label">{label}</span>
            <input
                class="jf__input"
                type="text"
                placeholder=placeholder
                prop:value=move || value.get()
                on:change=move |ev| on_commit.run(event_target_value(&ev))
            />
        </label>
    }
}

fn non_empty(s: &str) -> Option<String> {
    let t = s.trim();
    (!t.is_empty()).then(|| t.to_string())
}
