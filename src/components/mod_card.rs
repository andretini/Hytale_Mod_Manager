use dioxus::prelude::*;
use dioxus::events::MouseData;
use crate::api::ui_mod::{download_version_unified, search_exact_mod_unified};
use crate::api::local_mods::{InstallStatus, ModInstallInfo, check_install_status, install_mod, remove_mod};
use crate::api::settings::{AppSettings, ApiProvider};
use crate::api::ui_mod::UiMod;
use crate::state::mod_store::ModStore;

#[derive(PartialEq, Clone, Debug)]
pub enum ButtonAction {
    None,
    Install,
    Update,
    Remove
}

#[derive(PartialEq, Clone)]
pub struct ButtonState {
    pub text: &'static str,
    pub class: &'static str,
    pub disabled: bool,
    pub action: ButtonAction,
}

#[component]
pub fn ModCard(mod_data: ReadOnlySignal<UiMod>, onclick: EventHandler<UiMod>) -> Element {
    let mut app_settings = use_context::<Signal<AppSettings>>();
    let mut mod_store = use_context::<Signal<ModStore>>();

    let install_info = use_memo(move || {
        if mod_data().id == "0" {
            return ModInstallInfo {
                mod_name: mod_data().name.clone(),
                local_version: Some(mod_data().version.display_name.clone()),
                local_file_name: Some(mod_data().version.file_name.clone()),
                install_status: InstallStatus::Installed,
            };
        }

        let settings = app_settings.read();
        check_install_status(&settings, &mod_data().id, &mod_data().version.file_id)
    });

    let mut error_msg = use_signal(|| Option::<String>::None);

    let button_info = use_memo(move || {
        let store = mod_store.read();
        let current_error = error_msg();
        let info = install_info();

        let is_processing = store.is_processing(&mod_data().id);

        let target_action = match info.install_status {
            InstallStatus::Installed => ButtonAction::Remove,
            InstallStatus::NotInstalled => ButtonAction::Install,
            InstallStatus::Outdated => ButtonAction::Update,
        };

        if is_processing {
            ButtonState {
                text: "WORKING...",
                class: "btn-secondary",
                disabled: true,
                action: ButtonAction::None
            }
        } else if current_error.is_some() {
            ButtonState {
                text: "RETRY",
                class: "btn-danger",
                disabled: false,
                action: target_action
            }
        } else {
            ButtonState {
                text: match target_action {
                    ButtonAction::Remove => "REMOVE",
                    ButtonAction::Install => "INSTALL",
                    ButtonAction::Update => "UPDATE",
                    _ => "UNKNOWN",
                },
                class: match target_action {
                    ButtonAction::Remove => "btn-danger",
                    ButtonAction::Install => "btn-brand",
                    ButtonAction::Update => "btn-warning",
                    _ => "btn-secondary",
                },
                disabled: false,
                action: target_action,
            }
        }
    });

    let has_icon = !mod_data().icon.is_empty();

    let handle_action = move |e: Event<MouseData>| {
        e.stop_propagation();

        let folder_opt = app_settings.read().get_game_folder();
        let folder = match folder_opt {
            Some(p) => p,
            None => {
                error_msg.set(Some("No Game Folder Set".to_string()));
                return;
            }
        };

        let current_action = button_info().action;
        if let ButtonAction::None = current_action { return }

        let mod_id = mod_data().id.clone();
        let file_id = mod_data().version.file_id.clone();
        let file_name = mod_data().version.file_name.clone();
        let mod_name = mod_data().name.clone();
        let version_name = mod_data().version.display_name.clone();
        let version_data = mod_data().version.clone();
        let local_file_to_remove = install_info().local_file_name.clone();
        let provider = app_settings.read().api_provider.clone();

        let mut error_msg_clone = error_msg.clone();
        let mut settings_signal = app_settings.clone();

        mod_store.write().set_processing(&mod_id, true);
        error_msg.set(None);

        spawn(async move {
            match current_action {
                ButtonAction::Install | ButtonAction::Update => {
                    if current_action == ButtonAction::Update {
                        if let Some(old_file) = local_file_to_remove {
                            let mut settings = settings_signal.write();
                            match remove_mod(&folder, &old_file, &mut settings) {
                                Ok(_) => {},
                                Err(_) => error_msg_clone.set(Some("FAILED to remove old file".to_string())),
                            }
                        }
                    }

                    let mut final_version_data = version_data.clone();
                    if final_version_data.download_url.is_none() {
                        if mod_id == "0" {
                            error_msg_clone.set(Some("Cannot update local-only mod".to_string()));
                            mod_store.write().set_processing(&mod_id, false);
                            return;
                        }

                        let exact_find = search_exact_mod_unified(
                            &provider,
                            mod_name.to_string()
                        ).await;

                        match exact_find {
                            Ok(mod_found) => {
                                final_version_data.download_url = mod_found.version.download_url;
                            },
                            Err(e) => {
                                error_msg_clone.set(Some("Failed to resolve download URL".to_string()));
                                mod_store.write().set_processing(&mod_id, false);
                                return;
                            }
                        }
                    }

                    let download_res = {
                        let settings = settings_signal.read();
                        download_version_unified(&settings, &final_version_data).await
                    };

                    match download_res {
                        Ok((_, bytes)) => {
                            let mut settings = settings_signal.write();
                            match install_mod(
                                &folder,
                                &final_version_data.file_name,
                                &bytes,
                                mod_id.clone(),
                                mod_name.clone(),
                                file_id,
                                version_name.clone(),
                                provider,
                                &mut settings
                            ) {
                                Ok(_) => {},
                                Err(e) => {
                                    error_msg_clone.set(Some(format!("Install error: {}", e)));
                                }
                            }
                        }
                        Err(e) => {
                            error_msg_clone.set(Some(format!("Download failed: {}", e)));
                        }
                    }
                }
                ButtonAction::Remove => {
                    if let Some(local_name) = local_file_to_remove {
                        let mut settings = settings_signal.write();
                        match remove_mod(&folder, &local_name, &mut settings) {
                            Ok(_) => {},
                            Err(e) => {
                                error_msg_clone.set(Some(e));
                            }
                        }
                    }
                }
                _ => {},
            }

            mod_store.write().set_processing(&mod_id, false);
        });
    };

    rsx! {
        div {
            onclick: move |_| onclick.call(mod_data()),
            style: "cursor: pointer; background-color: var(--bg-tertiary); padding: 10px; border-radius: 8px; display: flex; align-items: center; gap: 15px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); transition: transform 0.1s;",

            div {
                style: "width: 50px; height: 50px; background-color: var(--bg-secondary); border-radius: 4px; overflow: hidden; display: flex; align-items: center; justify-content: center;",
                if has_icon {
                    img { src: "{mod_data().icon}", style: "width: 100%; height: 100%; object-fit: cover;" }
                } else {
                    span { style: "font-size: 10px; color: var(--text-secondary);", "IMG" }
                }
            }

            div { style: "flex: 1;",
                div { style: "display: flex; align-items: center; gap: 8px;",
                    span { style: "font-weight: bold; color: var(--text-primary);", "{mod_data().name}" }

                    {
                        let info = install_info();
                        let display_version = info.local_version.clone().unwrap_or_default();

                        match info.install_status {
                            InstallStatus::Installed => rsx! {
                                span {
                                    style: "font-size: 10px; background: var(--bg-secondary); color: var(--brand-primary); padding: 2px 6px; border-radius: 4px; border: 1px solid var(--success);",
                                    "✔ Installed"
                                },
                                span {
                                    style: "font-size: 10px; background: var(--bg-secondary); color: var(--text-secondary); padding: 2px 6px; border-radius: 4px; border: 1px solid var(--success);",
                                    "{display_version}"
                                }
                            },
                            InstallStatus::NotInstalled => rsx! { },
                            InstallStatus::Outdated => rsx! {
                                span {
                                    style: "font-size: 10px; background: var(--bg-secondary); color: var(--warning); padding: 2px 6px; border-radius: 4px; border: 1px solid var(--success);",
                                    "⚠ Outdated"
                                },
                                span {
                                    style: "font-size: 10px; background: var(--bg-secondary); color: var(--text-secondary); padding: 2px 6px; border-radius: 4px; border: 1px solid var(--success);",
                                    "{display_version}"
                                }
                            }
                        }
                    }
                }
                div { style: "font-size: 11px; color: var(--brand-primary); margin-bottom: 2px;", "By {mod_data().authors}" }
                if let Some(err) = error_msg() {
                    div { style: "font-size: 10px; color: var(--danger);", "{err}" }
                } else {
                    div { style: "font-size: 12px; color: var(--text-secondary);", "{mod_data().summary}" }
                }
            }

            button {
                class: "btn {button_info().class}",
                disabled: button_info().disabled,
                onclick: handle_action,
                "{button_info().text}"
            }
        }
    }
}