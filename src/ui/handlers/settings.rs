use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, error};
use slint::ComponentHandle;
use crate::{AppWindow, AppState, Theme, AppI18n, config, i18n};

pub fn setup(app: &AppWindow, app_handle: slint::Weak<AppWindow>, app_state: Arc<Mutex<AppState>>) {
    let ah = app_handle.clone();
    let as_ptr = app_state.clone();
    app.on_save_settings(move || {
        let ah = ah.clone();
        let as_ptr = as_ptr.clone();
        let _ = slint::spawn_local(async move {
            if let Some(app) = ah.upgrade() {
                let ui_language = app.get_ui_language().to_string();
                let distro_location = app.get_distro_location().to_string();
                let logs_location = app.get_logs_location().to_string();
                let auto_shutdown = app.get_auto_shutdown();
                let log_level = app.get_log_level() as u8;
                let log_days = app.get_log_days() as u8;
                let check_update = app.get_check_update_interval() as u8;
                
                let mut state = as_ptr.lock().await;
                let temp_location = state.config_manager.get_settings().temp_location.clone();
                let current_logs_location = state.config_manager.get_settings().logs_location.clone();

                // If log path or level changes, update logging system
                if let Some(ls) = state.logging_system.as_mut() {
                    if current_logs_location != logs_location {
                        ls.update_path(&logs_location);
                    }
                    ls.update_level(log_level);
                }

                // Update i18n
                let system_lang = state.config_manager.get_config().system.system_language.clone();
                let lang_to_load = if ui_language == "auto" {
                    &system_lang
                } else {
                    &ui_language
                };
                i18n::load_resources(lang_to_load);
                app.global::<AppI18n>().set_version(app.global::<AppI18n>().get_version() + 1);

                let user_settings = config::UserSettings {
                    modify_time: chrono::Utc::now().timestamp_millis().to_string(),
                    distro_location,
                    logs_location: logs_location.clone(),
                    temp_location,
                    ui_language,
                    auto_shutdown,
                    dark_mode: app.global::<Theme>().get_dark_mode(),
                    log_level,
                    log_days,
                    check_update,
                    check_time: state.config_manager.get_settings().check_time.clone(),
                };

                match state.config_manager.update_settings(user_settings) {
                    Ok(_) => {
                        drop(state);
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(app) = ah.upgrade() {
                                // Translate message if possible, or just keep english for now as it's dynamic
                                // But better to use a key if we had one "settings.saved_success"
                                app.set_current_message(i18n::t("settings.saved_success").into());
                                app.set_show_message_dialog(true);
                            }
                        });
                    }
                    Err(e) => {
                        let error_msg = i18n::tr("settings.saved_failed", &[e.to_string()]);
                        drop(state);
                        error!("{}", error_msg);
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(app) = ah.upgrade() {
                                app.set_current_message(error_msg.into());
                                app.set_show_message_dialog(true);
                            }
                        });
                    }
                }
            }
        });
    });

    let ah = app_handle.clone();
    app.on_select_distro_folder(move || {
        if let Some(path) = rfd::FileDialog::new()
            .set_title(i18n::t("settings.select_distro_dir"))
            .pick_folder()
        {
            if let Some(app) = ah.upgrade() {
                app.set_distro_location(path.display().to_string().into());
            }
        }
    });

    let ah = app_handle.clone();
    app.on_select_logs_folder(move || {
        if let Some(path) = rfd::FileDialog::new()
            .set_title(i18n::t("settings.select_log_dir"))
            .pick_folder()
        {
            if let Some(app) = ah.upgrade() {
                app.set_logs_location(path.display().to_string().into());
            }
        }
    });

    let ah = app_handle.clone();
    let as_ptr = app_state.clone();
    app.on_toggle_theme(move || {
        let ah = ah.clone();
        let as_ptr = as_ptr.clone();
        let _ = slint::spawn_local(async move {
            if let Some(app) = ah.upgrade() {
                let dark_mode = app.global::<Theme>().get_dark_mode();
                let mut state = as_ptr.lock().await;
                let mut settings = state.config_manager.get_settings().clone();
                settings.dark_mode = dark_mode;
                if let Err(e) = state.config_manager.update_settings(settings) {
                    error!("Failed to save color mode: {}", e);
                } else {
                    info!("Color mode saved: {}", if dark_mode { "Dark" } else { "Light" });
                }
            }
        });
    });
}
