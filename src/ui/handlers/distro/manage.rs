use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;
use crate::{AppWindow, AppState, i18n};
use crate::ui::data::refresh_distros_ui;

pub fn setup(app: &AppWindow, app_handle: slint::Weak<AppWindow>, app_state: Arc<Mutex<AppState>>) {
    // Terminal
    let ah = app_handle.clone();
    let as_ptr = app_state.clone();
    app.on_terminal_distro(move |name| {
        info!("Operation: Open terminal - {}", name);
        let ah = ah.clone();
        let as_ptr = as_ptr.clone();
        let _ = slint::spawn_local(async move {
            {
                let app_state = as_ptr.lock().await;
                let executor = app_state.wsl_dashboard.executor().clone();
                drop(app_state);
                let _ = executor.open_distro_terminal(&name).await;
            }
            // Refresh status immediately after opening terminal
            refresh_distros_ui(ah, as_ptr).await;
        });
    });

    // Folder
    let ah = app_handle.clone();
    let as_ptr = app_state.clone();
    app.on_folder_distro(move |name| {
        info!("Operation: Open folder - {}", name);
        let ah = ah.clone();
        let as_ptr = as_ptr.clone();
        let _ = slint::spawn_local(async move {
            {
                let app_state = as_ptr.lock().await;
                let executor = app_state.wsl_dashboard.executor().clone();
                drop(app_state);
                let _ = executor.open_distro_folder(&name).await;
            }
            refresh_distros_ui(ah, as_ptr).await;
        });
    });

    // VS Code
    let ah = app_handle.clone();
    let as_ptr = app_state.clone();
    app.on_vscode_distro(move |name| {
        info!("Operation: Open VS Code - {}", name);
        let ah = ah.clone();
        let as_ptr = as_ptr.clone();
        let _ = slint::spawn_local(async move {
            let ah_timer = ah.clone();
            if let Some(app) = ah.upgrade() {
                app.set_show_vscode_startup(true);
            }

            {
                let app_state = as_ptr.lock().await;
                let executor = app_state.wsl_dashboard.executor().clone();
                drop(app_state);
                let _ = executor.open_distro_vscode(&name).await;
            }
            refresh_distros_ui(ah, as_ptr).await;

            slint::Timer::single_shot(std::time::Duration::from_secs(6), move || {
                if let Some(app) = ah_timer.upgrade() {
                    if app.get_show_vscode_startup() {
                        app.set_show_vscode_startup(false);
                    }
                }
            });
        });
    });

    // Edit configuration
    let ah = app_handle.clone();
    let as_ptr = app_state.clone();
    app.on_edit_bashrc_distro(move |name| {
        info!("Operation: Edit .bashrc - {}", name);
        let ah = ah.clone();
        let as_ptr = as_ptr.clone();
        let _ = slint::spawn_local(async move {
            let app_state = as_ptr.lock().await;
            app_state.wsl_dashboard.open_distro_bashrc(&name).await;
            drop(app_state);
            refresh_distros_ui(ah, as_ptr).await;
        });
    });

    // Instance information
    let ah = app_handle.clone();
    let as_ptr = app_state.clone();
    app.on_information_clicked(move |name| {
        info!("Operation: Fetch information - {}", name);
        let ah = ah.clone();
        let as_ptr = as_ptr.clone();
        let name = name.to_string();
        let _ = slint::spawn_local(async move {
            if let Some(app) = ah.upgrade() {
                app.set_operation_text(i18n::t("operation.fetching_info").into());
                app.set_show_operation(true);
            }
            let result = {
                let app_state = as_ptr.lock().await;
                let executor = app_state.wsl_dashboard.executor().clone();
                drop(app_state);
                executor.get_distro_information(&name).await
            };
            if let Some(app) = ah.upgrade() {
                app.set_show_operation(false);
                if result.success {
                    if let Some(data) = result.data {
                        let mut slint_data = app.get_information();
                        slint_data.distro_name = data.distro_name.into();
                        slint_data.wsl_version = data.wsl_version.into();
                        slint_data.status = data.status.into();
                        slint_data.install_location = data.install_location.into();
                        slint_data.vhdx_path = data.vhdx_path.into();
                        slint_data.vhdx_size = data.vhdx_size.into();
                        slint_data.actual_used = data.actual_used.into();
                        app.set_information(slint_data);
                        app.set_show_information(true);
                    }
                } else {
                    let err = result.error.unwrap_or_else(|| i18n::t("dialog.error"));
                    app.set_current_message(i18n::tr("dialog.info_failed", &[err]).into());
                    app.set_show_message_dialog(true);
                }
            }
        });
    });
}
