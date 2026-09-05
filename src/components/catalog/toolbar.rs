use leptos::prelude::*;

/// Search box + tag-filter chips. Writes straight into the parent's signals.
#[component]
pub fn CatalogToolbar(
    search: RwSignal<String>,
    active_tags: RwSignal<Vec<String>>,
    #[prop(into)] all_tags: Signal<Vec<String>>,
) -> impl IntoView {
    view! {
        <div class="toolbar">
            <div class="toolbar__row">
                <input
                    class="toolbar__search"
                    type="text"
                    placeholder="Search for dashboards"
                    prop:value=search
                    on:input=move |ev| search.set(event_target_value(&ev))
                />
                <Show when=move || !active_tags.get().is_empty()>
                    <button class="toolbar__clear" on:click=move |_| active_tags.set(Vec::new())>
                        "Clear tags"
                    </button>
                </Show>
            </div>

            <Show when=move || !all_tags.get().is_empty() fallback=|| ()>
                <div class="toolbar__tags">
                    <For
                        each=move || all_tags.get()
                        key=|t: &String| t.clone()
                        children=move |tag| {
                            let for_active = tag.clone();
                            let for_click = tag.clone();
                            let is_active = move || active_tags.get().iter().any(|t| *t == for_active);
                            view! {
                                <button
                                    class="chip chip--toggle"
                                    class:is-active=is_active
                                    on:click=move |_| {
                                        let tag = for_click.clone();
                                        active_tags
                                            .update(|tags| {
                                                match tags.iter().position(|t| *t == tag) {
                                                    Some(i) => {
                                                        tags.remove(i);
                                                    }
                                                    None => tags.push(tag),
                                                }
                                            });
                                    }
                                >
                                    {tag.clone()}
                                </button>
                            }
                        }
                    />
                </div>
            </Show>
        </div>
    }
}
