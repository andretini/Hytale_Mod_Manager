use crate::api::ui_mod::{download_version_unified, get_mod_versions_unified, search_exact_mod_unified};
use crate::api::local_mods::{check_install_status, install_mod, remove_mod, InstallStatus};
use crate::api::settings::AppSettings;
use crate::api::ui_mod::{UiMod, UiModVersion};
use crate::components::mod_card::{ButtonAction, ButtonState};
use crate::state::mod_store::ModStore;
use dioxus::events::MouseData;
use dioxus::prelude::*;

#[component]
pub fn ModInfoDialog(mod_data: UiMod, on_close: EventHandler<()>) -> Element {
    let mod_data = use_signal(|| mod_data.clone());
    let mut app_settings = use_context::<Signal<AppSettings>>();
    let mut mod_store = use_context::<Signal<ModStore>>();

    let mut active_tab = use_signal(|| "overview");
    let mut displayed_versions = use_signal(|| vec![]);

    use_resource(use_reactive(&mod_data().id, move |id| async move {
        let settings = app_settings.read();
        if let Ok(versions) = get_mod_versions_unified(&settings, &id).await {
            displayed_versions.set(versions);
        }
    }));

    let is_processing = mod_store().is_processing(&mod_data().id);

    let install_info = use_memo(move || {
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

    let mod_id_for_versions = mod_data().id.clone();
    let mod_name_for_versions = mod_data().name.clone();

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
        let mod_id = mod_data().id.clone();
        let mod_name = mod_data().name.clone();

        let version_data = mod_data().version.clone();
        let file_id = version_data.file_id.clone();
        let file_name = version_data.file_name.clone();
        let version_name = version_data.display_name.clone();
        let download_url_str = version_data.download_url.clone();

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
                            Ok(_) => {}
                            Err(e) => error_msg_clone.set(Some(e)),
                        }
                    } else {
                        error_msg_clone.set(Some("File not found locally".to_string()));
                    }
                }
                _ => {}
            }
            mod_store.write().set_processing(&mod_id, false);
        });
    };

    let mut selected_image = use_signal(|| None::<String>);

    use_effect(move || {
        let urls = &mod_data().gallery_urls;
        selected_image.set(urls.first().cloned());
    });

    let current_gallery_image =
        selected_image().or_else(|| mod_data().gallery_urls.first().cloned());

    rsx! {
        div {
            style: "position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.8); display: flex; align-items: center; justify-content: center; z-index: 99; backdrop-filter: blur(2px);",
            onclick: move |_| on_close.call(()),

            div {
                style: "background-color: var(--bg-secondary); width: 80%; height: 80%; max-width: 1400px; display: flex; flex-direction: column; padding: 10px; border-radius: 12px; border: 1px solid var(--text-secondary); box-shadow: 0 10px 25px rgba(0,0,0,0.5); color: var(--text-primary); gap: 10px; padding: 0 30px; padding-top: 30px;",
                onclick: |e| e.stop_propagation(),

                div { style: "display: flex; gap: 20px; align-items: flex-start; background-color: var(--bg-secondary); padding: 0;",
                    if !mod_data().icon.is_empty() {
                        img { src: "{mod_data().icon}", style: "width: 80px; height: 80px; border-radius: 10px; object-fit: cover; background-color: var(--bg-tertiary);" }
                    }
                    div { style: "flex: 1;",
                        div { style: "display: flex; flex-direction: row; gap: 10px; align-items: center;",
                            h2 { style: "margin: 0;", "{mod_data().name}" },
                            {
                                let info = install_info();
                                let version_display = info.local_version.clone().unwrap_or_default();
                                rsx! {
                                    span {
                                        style: "font-size: 14px; background: var(--bg-quaternary); color: var(--text-secondary); padding: 4px 8px; border-radius: 4px; text-align: center; display: flex; align-items: center; justify-content: center; height: fit-content;",
                                        "{version_display}"
                                    }
                                }
                            }
                        },
                        div { style: "font-size: 14px; margin-top: 5px;", "By {mod_data().authors}" }
                    }
                    button { class: "btn btn-ghost", onclick: move |_| on_close.call(()), "X" }
                }

                {
                    if let InstallStatus::Outdated = install_info().install_status {
                        rsx! {
                            div {
                                style: "background: rgba(255, 165, 0, 0.2); color: var(--text-primary); padding: 12px; border-radius: 6px; font-size: 14px; font-weight: bold; text-align: center;",
                                span {"⚠ A newer version is available. Update to the latest version!"},
                            }
                        }
                    }
                    else {
                        rsx! {}
                    }
                }

                div { style: "padding: 0 30px; display: flex; gap: 10px; border-bottom: 1px solid var(--bg-tertiary);",
                    button {
                        class: if active_tab() == "overview" { "btn btn-tab-active" } else { "btn btn-tab" },
                        onclick: move |_| active_tab.set("overview"),
                        "Overview"
                    }
                    button {
                        class: if active_tab() == "versions" { "btn btn-tab-active" } else { "btn btn-tab" },
                        onclick: move |_| active_tab.set("versions"),
                        "Versions ({displayed_versions.read().len()})"
                    }
                }

                div { style: "flex: 1; overflow-y: auto; padding: 30px;",
                    if let Some(err) = error_msg() {
                        div { style: "color: var(--danger); margin-bottom: 20px;", "Error: {err}" }
                    }

                    if active_tab() == "overview" {
                        div {
                            style: "display: flex; flex-direction: column; gap: 20px;",
                            if !mod_data().banner.is_empty() {
                                div {
                                    style: "width: 100%; height: 200px; background-color: var(--bg-tertiary); border-radius: 8px; overflow: hidden; border: 1px solid var(--bg-secondary);",
                                    img {
                                        src: "{mod_data().banner}",
                                        style: "width: 100%; height: 100%; object-fit: cover;"
                                    }
                                }
                            }
                            div {
                                style: "margin: 0 10px; line-height: 1.6; color: var(--text-secondary); font-size: 14px;",
                                "{mod_data().summary}"
                            }
                            if !mod_data().gallery_urls.is_empty() {
                                div {
                                    style: "display: flex; flex-direction: column; gap: 10px; margin: 10px auto; width: 100%; max-width: 800px;",
                                    div {
                                        style: "width: 100%; aspect-ratio: 16/9; max-height: 40vh; border-radius: 8px; border: 1px solid var(--bg-secondary); display: flex; align-items: center; justify-content: center; overflow: hidden;",
                                        if let Some(img_src) = current_gallery_image {
                                            img { src: "{img_src}", style: "width: 100%; height: 100%; object-fit: contain;" }
                                        }
                                    }
                                    div {
                                        style: "display: flex; gap: 10px; overflow-x: auto; padding: 4px 2px; scrollbar-width: thin; justify-content: center;",
                                        for image in mod_data().gallery_urls.iter() {
                                            {
                                                let image_owned = image.clone();
                                                let is_selected = Some(image_owned.clone()) == selected_image() || (selected_image().is_none() && Some(image_owned.clone()) == mod_data().gallery_urls.first().cloned());
                                                let border_color = if is_selected { "var(--brand-primary)" } else { "transparent" };
                                                let opacity_val = if is_selected { "1.0" } else { "0.6" };
                                                rsx! {
                                                    img {
                                                        src: "{image_owned}",
                                                        onclick: move |_| selected_image.set(Some(image_owned.clone())),
                                                        style: "cursor: pointer; width: 80px; height: 50px; object-fit: cover; border-radius: 6px; flex-shrink: 0; transition: all 0.2s; border: 2px solid {border_color}; opacity: {opacity_val};"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else if active_tab() == "versions" {
                        div { style: "display: flex; flex-direction: column; gap: 10px;",
                            for version in displayed_versions.read().iter() {{
                                let mod_name_owned = mod_name_for_versions.clone();
                                let mod_id_owned = mod_id_for_versions.clone();
                                let version_data = version.clone();

                                rsx!{
                                    VersionRow {
                                        version: version.clone(),
                                        is_installed: {
                                            let current_settings = app_settings.read();
                                            if let Some(entry) = current_settings.installed_mods.values().find(|e| e.mod_id == mod_id_owned) {
                                                entry.file_id == version.file_id
                                            } else {
                                                false
                                            }
                                        },
                                        is_processing: is_processing,
                                        on_install: move |_| {
                                            let mod_id = mod_id_owned.clone();
                                            let mod_name = mod_name_owned.clone();
                                            let folder = match app_settings.read().get_game_folder() {
                                                Some(p) => p,
                                                None => { error_msg.set(Some("No Game Folder Set".to_string())); return; }
                                            };

                                            let file_id = version_data.file_id.clone();
                                            let file_name = version_data.file_name.clone();
                                            let version_name = version_data.display_name.clone();
                                            let provider = app_settings.read().api_provider.clone();

                                            let mut error_msg_clone = error_msg.clone();
                                            let mut settings_signal = app_settings.clone();
                                            let version_clone_for_dl = version_data.clone();

                                            (mod_store.write()).set_processing(&mod_id, true);
                                            error_msg.set(None);

                                            spawn(async move {
                                                let download_res = {
                                                    let settings = settings_signal.read();
                                                    download_version_unified(&settings, &version_clone_for_dl).await
                                                };

                                                match download_res {
                                                    Ok((_, bytes)) => {
                                                        let mut settings = settings_signal.write();
                                                        match install_mod(
                                                            &folder,
                                                            &file_name,
                                                            &bytes,
                                                            mod_id.clone(),
                                                            mod_name,
                                                            file_id,
                                                            version_name,
                                                            provider,
                                                            &mut settings
                                                        ) {
                                                            Ok(_) => {},
                                                            Err(e) => error_msg_clone.set(Some(e)),
                                                        }
                                                    }
                                                    Err(e) => error_msg_clone.set(Some(e)),
                                                }
                                                (mod_store.write()).set_processing(&mod_id, false);
                                            });
                                        }
                                    }
                                }
                            }}
                        }
                    }
                }

                div {
                    style: "display: flex; justify-content: space-between; align-items: center; padding: 20px 30px; border-top: 1px solid var(--bg-tertiary); background-color: var(--bg-secondary);",

                    button {
                        class: "btn {button_info().class}",
                        style: "z-index: 2; min-width: 120px;",
                        disabled: button_info().disabled,
                        onclick: handle_action,
                        "{button_info().text}"
                    }

                    button {
                        class: "btn btn-outline",
                        style: "display: flex; align-items: center; gap: 8px;",
                        onclick: move |_| {
                            let url = mod_data().website_url.clone();
                            spawn(async move {
                                if let Err(e) = open::that(&url) {
                                    error_msg.set(Some(format!("Could not open browser: {}", e)));
                                }
                            });
                        },
                        span { "View on Website ↗" }
                    }
                }
            }
        }
    }
}

#[component]
fn VersionRow(
    version: UiModVersion,
    is_installed: bool,
    is_processing: bool,
    on_install: EventHandler<UiModVersion>,
) -> Element {
    let (type_text, type_color) = match version.release_type {
        1 => ("R", "var(--success)"),
        2 => ("B", "var(--brand-primary)"),
        3 => ("A", "var(--danger)"),
        _ => ("?", "var(--text-secondary)"),
    };

    let has_url = version.download_url.is_some();

    let (btn_text, btn_class, btn_disabled) = if is_installed {
        ("Installed", "btn btn-ghost", true)
    } else if !has_url {
        ("Not Available", "btn btn-secondary", true)
    } else {
        ("Install", "btn btn-brand", false)
    };

    rsx! {
        div {
            style: "display: flex; align-items: center; background-color: var(--bg-tertiary); padding: 10px; border-radius: 6px; gap: 15px;",
            div {
                style: "background-color: {type_color}; color: white; width: 24px; height: 24px; display: flex; align-items: center; justify-content: center; border-radius: 4px; font-weight: bold; font-size: 12px;",
                "{type_text}"
            }
            div { style: "flex: 1; display: flex; flex-direction: column;",
                span { style: "color: var(--text-primary); font-weight: bold;", "{version.display_name}" }
                div { style: "font-size: 12px; color: var(--text-secondary); display: flex; gap: 10px;",
                    span { "{version.upload_date}" }
                    span { "•" }
                    span { "{version.game_versions.join(\", \")}" }
                }
            }
            button {
                class: "{btn_class}",
                style: "padding: 5px 15px; font-size: 12px;",
                disabled: btn_disabled || is_processing,
                onclick: move |_| {
                    if has_url {
                        on_install.call(version.clone())
                    }
                },
                "{btn_text}"
            }
        }
    }
}