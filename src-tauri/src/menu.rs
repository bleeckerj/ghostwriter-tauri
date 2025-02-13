use tauri::{
    AppHandle,
    Manager,
    Runtime,
    menu::{
        MenuBuilder, 
        MenuItemBuilder, 
        Submenu, 
        SubmenuBuilder,
        MenuEvent,
        Menu,
    }
 };

use crate::SimpleLog;
use crate::AppState;
use tauri_plugin_dialog::{DialogExt, Dialog, FileDialogBuilder};

 // Constants for menu IDs
 pub const MENU_FILE_NEW: &str = "file-new";
 pub const MENU_FILE_QUIT: &str = "file-quit";
 
 // Canon menu IDs 
 pub const MENU_CANON_LIST: &str = "canon-list";
 pub const MENU_CANON_NEW: &str = "canon-new";
 pub const MENU_CANON_LOAD: &str = "canon-load";
 pub const MENU_CANON_INGEST: &str = "canon-ingest";
 
 pub fn build_app_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Submenu<R>> {
    let quit_item = MenuItemBuilder::new("Quit")
        .id(MENU_FILE_QUIT)
        .accelerator("CmdOrControl+Q")
        .build(app)?;
 
    SubmenuBuilder::new(app, "App")
        .item(&quit_item)
        .build()
 }
 
 pub fn build_file_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Submenu<R>> {
    let new_item = MenuItemBuilder::new("New")
        .id(MENU_FILE_NEW)
        .accelerator("CmdOrControl+N")
        .build(app)?;
 
    SubmenuBuilder::new(app, "File")
        .item(&new_item)
        .build()
 }
 
 pub fn build_canon_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Submenu<R>> {
    let list_item = MenuItemBuilder::new("List")
        .id(MENU_CANON_LIST)
        .accelerator("CmdOrControl+L")
        .build(app)?;
 
    let new_item = MenuItemBuilder::new("New")
        .id(MENU_CANON_NEW)
        .accelerator("CmdOrControl+Shift+N")
        .build(app)?;
 
    let load_item = MenuItemBuilder::new("Load")
        .id(MENU_CANON_LOAD)
        .accelerator("CmdOrControl+O")
        .build(app)?;
 
    let ingest_item = MenuItemBuilder::new("Ingest")
        .id(MENU_CANON_INGEST)
        .accelerator("CmdOrControl+I")
        .build(app)?;
 
    SubmenuBuilder::new(app, "Canon")
        .item(&list_item)
        .item(&new_item)
        .item(&load_item)
        .item(&ingest_item)
        .build()
 }
 
 pub fn build_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let app_menu = build_app_menu(app)?;
    let file_menu = build_file_menu(app)?;
    let canon_menu = build_canon_menu(app)?;
 
    MenuBuilder::new(app)
        .item(&app_menu)
        .item(&file_menu)
        .item(&canon_menu)
        .build()
 }
 
 pub fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, event: MenuEvent) {
    let app_state = app.state::<AppState>();

    match event.id.0.as_str() {
        MENU_FILE_NEW => {
            if let Ok(logger) = app_state.logger.lock() {
                //logger.log(SimpleLog::new("New file", "info"));
                println!("Need to refactor logger - New file");
            }
        }
        MENU_FILE_QUIT => {
            app.exit(0);
        }
        MENU_CANON_LIST => {
            // Handle canon list
        }
        MENU_CANON_NEW => {
            // Handle new canon
        }
        MENU_CANON_LOAD => {
            // Handle load canon
            println!("Load canon");
        }
        MENU_CANON_INGEST => {
            // Handle ingest canon
            let app_handle = app.clone();
            let dialog: &Dialog<R> = app.dialog();
            let state = app_state.clone();
        }
        _ => {}
    }
 }