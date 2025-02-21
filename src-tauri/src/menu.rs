use pdfium_render::prelude::embedded_destination;
use serde::Serialize;
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
use std::path::Path;
use tauri::Emitter;
use serde_json::json;
use std::sync::Arc;

 // Constants for menu IDs
 pub const MENU_FILE_SAVE: &str = "file-save";
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
    let new_item = MenuItemBuilder::new("Save")
        .id(MENU_FILE_SAVE)
        .accelerator("CmdOrControl+S")
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
 
 pub fn handle_menu_event<R: Runtime>(app: &AppHandle<R>,   event: MenuEvent) {
    let app_state = app.state::<AppState>();

    match event.id.0.as_str() {
        MENU_FILE_SAVE => {
            println!("New file");
            app.emit("save-file-dialog", json!({
                "defautPath": "new_file.txt",
                "filters": [
                    {
                        "name": "Text Files",
                        "extensions": ["txt", "md", "mdx"]
                    }
                ]}
            ));
        }

        MENU_FILE_QUIT => {
            app.exit(0);
        }
        MENU_CANON_LIST => {
            let doc_store = Arc::clone(&app_state.doc_store);
            let app_handle = app.clone();
            tauri::async_runtime::spawn(async move {
                match doc_store.fetch_documents().await {  // ✅ Ensure `fetch_documents` is async
                    Ok(listing) => {
                        // Serialize the listing to JSON
                        match serde_json::to_string(&listing) {
                            Ok(json_string) => {
                                // Emit the JSON string
                                // Send to the front end, basically.
                                app_handle.emit("canon-list", json_string);
                            }
                            Err(e) => {
                                // Handle serialization error
                                eprintln!("Error serializing listing to JSON: {}", e);
                                app_handle.emit("rich-log-message", json!({
                                    "message": "Error serializing document listing",
                                    "data": e.to_string(),
                                    "timestamp": chrono::Local::now().to_rfc3339(),
                                    "level": "error"
                                }));
                            }
                        }
                        // for doc in listing.documents {
                        //     let _ = app_handle.emit("rich-log-message", json!({
                        //         "message": format!("{}", doc.name),
                        //         "timestamp": chrono::Local::now().to_rfc3339(),
                        //         "level": "info"
                        //     }));
                        // }
                    }
                    Err(e) => {
                        let _ = app_handle.emit("rich-log-message", json!({
                            "message": "Error Listing Documents",
                            "data": e.to_string(),
                            "timestamp": chrono::Local::now().to_rfc3339(),
                            "level": "error"
                        }));
                    }
                }
            });
            
        }
        MENU_CANON_NEW => {
            // Handle new canon
            let app_handle = app.clone();
            let simple_log_data = SimpleLog {
                message: format!("{}", "New Canon feature not yet implemented, sadly.."),
                level: "info".to_string(),
                timestamp: chrono::Local::now().to_rfc3339().to_string(),
                id: None,
            };
            let _ = app_handle.emit("simple-log-message", simple_log_data);
        }
        MENU_CANON_LOAD => {
            // Handle load canon
            let app_handle = app.clone();
            let simple_log_data = SimpleLog {
                message: format!("{}", "Load Canon feature not yet implemented, sadly.."),
                level: "info".to_string(),
                timestamp: chrono::Local::now().to_rfc3339().to_string(),
                id: None,
            };
            let _ = app_handle.emit("simple-log-message", simple_log_data);
        }
        MENU_CANON_INGEST => {
            // Handle ingest canon
            println!("Ingest canon");
            app.emit("open-file-dialog-for-ingest", ());
        }
        _ => {}
    }
 }