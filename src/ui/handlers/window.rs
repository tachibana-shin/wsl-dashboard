use slint::ComponentHandle;
use crate::AppWindow;

pub fn setup(app: &AppWindow, app_handle: slint::Weak<AppWindow>) {
    let ah = app_handle.clone();
    app.on_window_close(move || {
        if let Some(app) = ah.upgrade() {
            let _ = app.hide();
        }
    });

    let ah = app_handle.clone();
    app.on_window_minimize(move || {
        if let Some(app) = ah.upgrade() {
            app.window().set_minimized(true);
        }
    });

    let ah = app_handle.clone();
    app.on_window_maximize(move || {
        if let Some(app) = ah.upgrade() {
            let is_max = app.get_is_maximized();
            app.set_is_maximized(!is_max);
            app.window().set_maximized(!is_max);
        }
    });

    let ah = app_handle.clone();
    app.on_window_drag_delta(move |dx, dy| {
        if let Some(app) = ah.upgrade() {
            let pos = app.window().position();
            app.window().set_position(slint::PhysicalPosition::new(
                pos.x + dx as i32,
                pos.y + dy as i32
            ));
        }
    });
}
