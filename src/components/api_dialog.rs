use dioxus::prelude::*;
use crate::api::settings::{AppSettings, ApiProvider};
use crate::components::drop_down::ThemeDropdown;

#[component]
pub fn ApiDialog(on_close: EventHandler<()>) -> Element {
    let mut app_settings = use_context::<Signal<AppSettings>>();

    let current_settings = app_settings.read();
    let mut selected_provider = use_signal(|| current_settings.api_provider.clone());
    let mut input_val = use_signal(|| current_settings.api_key.clone().unwrap_or_default());

    let handle_save = move |_| {
        let key = input_val();
        let provider = selected_provider();
        app_settings.write().change_api(provider, key);
        let _ = app_settings.read().save();
        on_close.call(());
    };

    rsx! {
        div {
            style: "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.7); display: flex; align-items: center; justify-content: center; z-index: 99;",
            onclick: move |_| on_close.call(()),

            div {
                style: "background-color: var(--bg-tertiary); width: 400px; padding: 25px; border-radius: 10px; display: flex; flex-direction: column; gap: 15px; border: 1px solid var(--border-color); box-shadow: 0 4px 15px rgba(0,0,0,0.5);",
                onclick: |e| e.stop_propagation(),

                h3 { style: "margin: 0; color: var(--text-primary);", "Configure API" }
                p { style: "margin: 0; font-size: 10px; color: var(--text-secondary);", "The API key may take a few Hours to start working after generating it on the website" }

                div { style: "display: flex; flex-direction: column; gap: 5px;",
                    label { style: "font-size: 12px; color: var(--text-secondary);", "Provider" }
                    ThemeDropdown {
                        items: ["ModTale".to_string(), "CurseForge".to_string()],
                        upwards: false,
                        placeholder: "Api Provider",
                        default_index: Some(selected_provider().clone() as usize),
                        on_select: move |idx| {
                            match idx {
                                0 => selected_provider.set(ApiProvider::Modtale),
                                1 => selected_provider.set(ApiProvider::CurseForge),
                                _ => {}
                            }
                        },
                    }
                }

                div { style: "display: flex; flex-direction: column; gap: 5px;",
                    label { style: "font-size: 12px; color: var(--text-secondary);", "API Key" }
                    input {
                        r#type: "password",
                        style: "padding: 8px; border-radius: 5px; background: var(--bg-secondary); color: var(--text-primary); border: 1px solid var(--border-color);",
                        placeholder: "Paste API Key here...",
                        value: "{input_val}",
                        oninput: move |e| input_val.set(e.value())
                    }
                    div { style: "font-size: 10px; color: var(--text-secondary); margin-top: 2px;",
                        if selected_provider() == ApiProvider::CurseForge {
                            "Requires an API Key from the CurseForge Console."
                        } else {
                            "ModTale key (Optional for public access)."
                        }
                    }
                }

                div { style: "display: flex; gap: 10px; margin-top: 10px;",
                    button {
                        class: "btn btn-ghost",
                        style: "flex: 1; background-color: var(--bg-quaternary);",
                        onclick: move |_| on_close.call(()),
                        "Cancel"
                    }
                    button {
                        class: "btn btn-brand",
                        style: "flex: 1;",
                        onclick: handle_save,
                        "Save & Apply"
                    }
                }
            }
        }
    }
}