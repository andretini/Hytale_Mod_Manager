use dioxus::prelude::*;
use crate::api::ui_mod::search_mods_unified;
use crate::api::local_mods::{check_install_status, InstallStatus};
use crate::api::ui_mod::UiMod;
use crate::api::settings::AppSettings;
use crate::components::drop_down::ThemeDropdown;
use crate::components::mod_card::ModCard;
use crate::state::mod_store::ModStore;

const PAGE_SIZE: u32 = 20;

#[component]
pub fn SearchPage(on_open_info: EventHandler<UiMod>) -> Element {
    let mut mod_store = use_context::<Signal<ModStore>>();
    let app_settings = use_context::<Signal<AppSettings>>();

    let mut query = use_signal(|| String::new());
    let mut search_sort = use_signal(|| 0);
    let mut page_index = use_signal(|| 0);
    let mut search_trigger = use_signal(|| 0);

    let search_resource = use_resource(move || async move {
        let _ = search_trigger();
        let current_idx = page_index();
        let query = query.peek().clone();
        let settings = app_settings.read().clone();
        let offset = current_idx * PAGE_SIZE;
        let sort = search_sort();
        
        match search_mods_unified(&settings, sort, query, offset).await {
            Ok((ui_mods, total)) => (ui_mods, total),
            Err(_) => (vec![], 0)
        }
    });

    let (mods, total_pages) = match search_resource.read().as_ref() {
        Some((m, t)) => (m.clone(), *t),
        None => (vec![], 0)
    };

    let current_page_display = page_index() + 1;

    use_effect(move || {
        if let Some((mods, _)) = search_resource.read().as_ref() {
            let settings = app_settings.read();

            for m in mods {
                let install_info = check_install_status(&settings, &m.id, &m.version.file_id);

                if install_info.install_status != InstallStatus::NotInstalled {
                    mod_store.write().set_info(&m.id, install_info);
                }
            }
        }
    });


    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100%;",

            h2 { style: "color: var(--text-primary); margin-top: 0;", "Search Mods" }

            div {
                style: "display: flex; gap: 10px; margin-bottom: 20px;",
                input {
                    style: "flex: 5;",
                    placeholder: "Search mods...",
                    value: "{query}",
                    oninput: move |e| {
                        query.set(e.value());
                        page_index.set(0);
                    },
                    onkeydown: move |e| {
                        if e.key() == Key::Enter {
                            search_trigger += 1;
                        }
                    }
                }

                div { style: "flex: 2; min-width: 0;",
                    ThemeDropdown {
                        items: ["Popularity".to_string(), "Featured".to_string(), "LastUpdated".to_string(), "Rating".to_string(), "Creation Date".to_string(), "Total Downloads".to_string()],
                        upwards: false,
                        placeholder: "Theme",
                        default_index: Some(0),
                        on_select: move |c: usize| search_sort.set(c as u32),
                    }
                }
                button { class: "btn btn-brand", onclick: move |_| *search_trigger.write() += 1, "Search" }
            }

            div {
                style: "flex: 1; overflow-y: auto; padding-right: 5px; margin-bottom: 10px;",

                if search_resource.finished() {
                    if mods.is_empty() {
                         div { style: "text-align: center; color: var(--text-secondary); margin-top: 50px;", "No results found." }
                    }
                    div {
                        style: "display: flex; flex-direction: column; gap: 10px;",
                        for m in mods {
                            ModCard { key: "{m.id}", mod_data: m, onclick: on_open_info }
                        }
                    }
                } else {
                    div { style: "text-align: center; color: var(--text-secondary); margin-top: 50px;", "Searching..." }
                }
            }

            div {
                style: "display: flex; justify-content: center; align-items: center; gap: 20px; padding: 5px; background-color: var(--bg-tertiary); border-radius: 8px; border: 1px solid var(--border-color); margin-top: auto;",

                button {
                    class: "btn btn-ghost",
                    style: "border-radius: 50%; width: 32px; height: 32px; padding: 0; display: flex; align-items: center; justify-content: center;",
                    disabled: page_index() == 0,
                    onclick: move |_| page_index -= 1,
                    "❮"
                }

                span {
                    style: "color: var(--text-primary); font-size: 14px; font-weight: bold;",
                    "Page {current_page_display} of {total_pages}"
                }

                button {
                    class: "btn btn-ghost",
                    style: "border-radius: 50%; width: 32px; height: 32px; padding: 0; display: flex; align-items: center; justify-content: center;",
                    disabled: current_page_display >= total_pages,
                    onclick: move |_| page_index += 1,
                    "❯"
                }
            }
        }
    }
}