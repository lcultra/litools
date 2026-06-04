use tauri::{App, Manager, image::Image, menu::Menu, menu::MenuItem, tray::TrayIconBuilder};

use crate::{state::AppState, window};

const SHOW_ID: &str = "show";
const SETTINGS_ID: &str = "settings";
const DIAGNOSTICS_ID: &str = "diagnostics";
const QUIT_ID: &str = "quit";

pub fn setup_tray(app: &App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, SHOW_ID, "Show litools", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, SETTINGS_ID, "Settings", true, None::<&str>)?;
    let diagnostics = MenuItem::with_id(app, DIAGNOSTICS_ID, "Diagnostics", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, QUIT_ID, "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &settings, &diagnostics, &quit])?;

    let mut tray = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id().as_ref() {
            SHOW_ID => {
                if let Some(window) = window::main_window(app) {
                    window::show_view(&window, "palette");
                }
            }
            SETTINGS_ID => {
                if let Some(window) = window::main_window(app) {
                    window::show_view(&window, "settings");
                }
            }
            DIAGNOSTICS_ID => {
                if let Some(window) = window::main_window(app) {
                    window::show_view(&window, "diagnostics");
                }
            }
            QUIT_ID => {
                if let Some(state) = app.try_state::<AppState>() {
                    state.request_quit();
                }
                app.exit(0);
            }
            _ => {}
        });

    let icon = Image::from_path(concat!(env!("CARGO_MANIFEST_DIR"), "/icons/icon-tray.png"))?;
    tray = tray.icon(icon);

    tray.build(app)?;

    Ok(())
}
