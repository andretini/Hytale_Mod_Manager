#![allow(non_snake_case)]

pub mod api;
pub mod components;
pub mod pages;
pub mod style;
mod state;

use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;
use crate::api::settings::{AppSettings, AppTheme};
use crate::api::ui_mod::UiMod;
use crate::components::sidebar::Sidebar;
use crate::components::mod_info::ModInfoDialog;
use crate::components::api_dialog::ApiDialog;
use crate::pages::search::SearchPage;
use crate::pages::installed::InstalledPage;
use crate::state::mod_store::ModStore;

#[derive(Clone, PartialEq)]
pub enum ActiveModal {
    None,
    ApiKey,
    ModInfo(UiMod),
}

#[derive(Clone, PartialEq)]
pub enum SidebarTab {
    SearchMods,
    Installed,
}



fn main() {
    let window = WindowBuilder::new()
        .with_title("My App Name")
        .with_resizable(true);

    let dir = std::env::temp_dir().join("com.hytale.modmanager.desktop.cache");

    let config = Config::new()
        .with_window(window)
        .with_menu(None)
        .with_data_directory(&dir)
        .with_custom_event_handler({
            let dir = dir.clone();
            move |event, _target| {
                use dioxus::desktop::tao::event::{Event, WindowEvent};

                if let Event::WindowEvent { event, .. } = event {
                    if let WindowEvent::CloseRequested = event {
                        println!("Cleaning cache...");
                        match std::fs::remove_dir_all(&dir) {
                            Ok(_) => println!("Cache removed"),
                            Err(e) => println!("Failed to remove cache: {e}"),
                        }
                    }
                }
            }
        })
        .with_disable_context_menu(false);

    LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(App);
}

fn App() -> Element {
    let settings_store = use_signal(|| AppSettings::load());
    let mod_store = use_signal(|| ModStore::new());

    use_context_provider(|| mod_store);
    use_context_provider(|| settings_store);

    let active_tab = use_signal(|| SidebarTab::SearchMods);
    let mut active_modal = use_signal(|| ActiveModal::None);


    use_effect(move || {
        let config = settings_store;

        if let Err(e) = config().save() {
            println!("Failed to save settings: {}", e);
        } else {
            println!("Settings saved.");
        }
    });

    let theme_class = match settings_store().get_theme() {
        AppTheme::Dark => {"dark-theme"}
        AppTheme::Light => {"light-theme"}
    } ;

    rsx! {
        style { "{style::GLOBAL_CSS}" }

        div {
            class: "{theme_class}",
            style: "display: flex; height: 100vh; width: 100vw;",

            Sidebar {
                active_tab: active_tab,
                on_open_api: move |_| active_modal.set(ActiveModal::ApiKey),
            }

            div {
                style: "flex: 1; background-color: var(--bg-secondary); padding: 10px; overflow-y: auto;",

                match active_tab() {
                    SidebarTab::SearchMods => rsx! {
                        SearchPage {
                            on_open_info: move |m: UiMod| active_modal.set(ActiveModal::ModInfo(m))
                        }
                    },
                    SidebarTab::Installed => rsx! {
                        InstalledPage {
                            on_open_info: move |m: UiMod| active_modal.set(ActiveModal::ModInfo(m))
                        }
                    }
                }
            }

            match active_modal() {
                ActiveModal::ApiKey => rsx! {
                    ApiDialog { on_close: move |_| active_modal.set(ActiveModal::None) }
                },
                ActiveModal::ModInfo(m) => rsx! {
                    ModInfoDialog { 
                        mod_data: m, 
                        on_close: move |_| active_modal.set(ActiveModal::None) 
                    }
                },
                ActiveModal::None => rsx! {}
            }
        }
    }
}