use crate::components::chat_context::ChatContext;
use crate::cx_id::{CxIdValue, request};
use leptos::prelude::*;
use leptos_fluent::move_tr;
use uuid::Uuid;

#[component]
pub fn CxIdEditor(
    node_id: Uuid,
    cx_id: Signal<Option<String>>,
    #[prop(optional)] object: bool,
) -> impl IntoView {
    let ctx = expect_context::<ChatContext>();
    let popover_id = format!("cx-id-editor-{node_id}");
    let open_target = popover_id.clone();
    let close_target = popover_id.clone();
    let draft = RwSignal::new(String::new());
    let ready = RwSignal::new(false);
    let pending = RwSignal::new(false);
    let saved = RwSignal::new(false);
    let error = RwSignal::new(None::<u16>);

    let open = move |event: web_sys::MouseEvent| {
        event.stop_propagation();
        if pending.get_untracked() {
            return;
        }
        pending.set(true);
        ready.set(false);
        saved.set(false);
        error.set(None);
        leptos::task::spawn_local(async move {
            match request::<CxIdValue>(&format!("/api/cx-id/{node_id}"), "GET", None::<&CxIdValue>)
                .await
            {
                Ok(value) => {
                    draft.try_set(value.cx_id.unwrap_or_default());
                    ready.try_set(true);
                }
                Err(status) => {
                    error.try_set(Some(status));
                }
            }
            pending.try_set(false);
        });
    };

    let save = move |event: web_sys::SubmitEvent| {
        event.prevent_default();
        event.stop_propagation();
        if pending.get_untracked() || !ready.get_untracked() {
            return;
        }
        let id = draft.get_untracked();
        let value = CxIdValue {
            cx_id: (!id.is_empty()).then_some(id),
        };
        pending.set(true);
        saved.set(false);
        error.set(None);
        leptos::task::spawn_local(async move {
            match request::<CxIdValue>(&format!("/api/cx-id/{node_id}"), "PUT", Some(&value)).await
            {
                Ok(value) => {
                    ctx.remember_cx_id_edit(node_id, value.cx_id.clone());
                    draft.try_set(value.cx_id.unwrap_or_default());
                    saved.try_set(true);
                }
                Err(status) => {
                    error.try_set(Some(status));
                }
            }
            pending.try_set(false);
        });
    };

    let label = move || {
        let name = if object {
            move_tr!("cx-id-object").get()
        } else {
            "CX ID".to_string()
        };
        cx_id
            .get()
            .map_or_else(|| name.clone(), |id| format!("{name}: {id}"))
    };
    view! {
        <button type="button" class="cx-id-trigger" popovertarget=open_target
            popovertargetaction="show" title=label
            aria-label=move || format!("{}: {}", move_tr!("cx-id-edit").get(), label()) on:click=open>
            {label}
        </button>
        <div id=popover_id popover class="cx-id-popover">
            <form on:submit=save>
                <label>
                    {move_tr!("cx-id-optional")}
                    <textarea rows=3 autofocus=true prop:value=draft
                        disabled=move || pending.get() || !ready.get()
                        on:input=move |event| {
                            draft.set(event_target_value(&event));
                            saved.set(false);
                        }
                    />
                </label>
                <div class="cx-id-actions">
                    <button type="submit" disabled=move || pending.get() || !ready.get()>
                        {move_tr!("cx-id-save")}
                    </button>
                    <button type="button" disabled=move || pending.get() || !ready.get()
                        on:click=move |_| {
                            draft.set(String::new());
                            saved.set(false);
                        }>
                        {move_tr!("cx-id-clear")}
                    </button>
                    <button type="button" popovertarget=close_target popovertargetaction="hide">
                        {move_tr!("cx-id-close")}
                    </button>
                </div>
                <Show when=move || saved.get()>
                    <div role="status">{move_tr!("cx-id-saved")}</div>
                </Show>
                <Show when=move || pending.get()>
                    <div role="status">{move_tr!("cx-id-working")}</div>
                </Show>
                {move || error.get().map(|status| {
                    view! {
                        <div role="alert">
                            {match status {
                                401 => move_tr!("cx-id-session-error").get(),
                                403 | 404 => move_tr!("cx-id-access-error").get(),
                                400 => move_tr!("cx-id-invalid").get(),
                                _ => move_tr!("cx-id-request-error").get(),
                            }}
                        </div>
                    }
                })}
            </form>
        </div>
    }
}
