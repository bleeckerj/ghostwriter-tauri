use pdfium_render::prelude::embedded_destination;
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
            if let Ok(doc_store) = app_state.doc_store.lock() {
                match doc_store.fetch_documents() {
                    Ok(listing) => {
                        // Emit overview message
                        let _ = app.emit("rich-log-message", json!({
                            "message": format!(
                                "Canon: {} File: {})",
                                listing.canon_name,
                                listing.canon_file
                            ),
                            "data": "",
                            "timestamp": chrono::Local::now().to_rfc3339(),
                            "level": "info"
                        }));

                        // Emit each document as a separate rich log entry
                        for doc in listing.documents {
                            let _ = app.emit("rich-log-message", json!({
                                "message": format!(
                                    "{}",
                                    doc.name),
                                "timestamp": chrono::Local::now().to_rfc3339(),
                                "level": "info"
                            }));
                        }
                    },
                    Err(e) => {
                        let _ = app.emit("rich-log-message", json!({
                            "message": "Error Listing Documents",
                            "data": e.to_string(),
                            "timestamp": chrono::Local::now().to_rfc3339(),
                            "level": "error"
                        }));
                    }
                }
            }
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
            let dialog: Dialog<R> = app_handle.dialog().clone();
            let state = app_state.clone();
            let embedding_generator = state.embedding_generator.clone();
            let doc_store = state.doc_store.clone(); // ✅ Clone Arc<Mutex<...>>

            FileDialogBuilder::new(dialog)
        .add_filter("Text Files", &["txt", "md", "rs"])
        .pick_file(move |file_path| {
            println!("File path: {:?}", file_path);
            if let Some(f_path) = file_path {
                println!("f_path {:?}", f_path);
                let p_path = f_path.as_path().unwrap();
                let file_name = p_path.file_name();
                println!("file_name {:?}", file_name);
                println!("p_path {:?}", p_path);
            // let mut doc_store = doc_store.lock().unwrap(); // Lock to mutate
            // doc_store.process_document(p_path, &embedding_generator);
            }
            // if let Some(path) = file_path {
            //     let embedding_generator = state.embedding_generator.clone();
            //     let path_buf = path.clone(); // ✅ path is already a PathBuf
            //     let file_name = path_buf.file_name()
            //         .unwrap_or_default()
            //         .to_string_lossy()
            //         .to_string();
                
            //     // ✅ Clone necessary data for async
            //     let path_for_async = path_buf.clone();
            //     let app_handle = app_handle.clone();
            //     let doc_store = state.doc_store.clone();
            // }
        });
        }
        _ => {}
    }
 }