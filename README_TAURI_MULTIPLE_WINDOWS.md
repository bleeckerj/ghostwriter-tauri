# Creating Multiple Windows in Tauri Applications

## Instructions for LLM Implementation

This guide provides step-by-step instructions for implementing multiple windows in a Tauri application. Follow these patterns to avoid popup blocking and ensure proper window management.

## Overview

Tauri supports creating additional windows beyond the main application window. These are native OS windows, not browser popups, so they bypass popup blocking entirely. Windows can be created from either the Rust backend (recommended) or JavaScript frontend.

## Method 1: Rust Backend (Recommended)

### 1. Add Required Imports
```rust
use tauri::{WebviewWindowBuilder, WebviewUrl, AppHandle, Manager};
```

### 2. Create Window Command
```rust
#[tauri::command]
async fn open_secondary_window(app_handle: AppHandle, window_id: String, page_path: String) -> Result<(), String> {
    // Check if window already exists to prevent duplicates
    if let Some(existing_window) = app_handle.get_webview_window(&window_id) {
        // Focus existing window instead of creating new one
        existing_window.set_focus().map_err(|e| format!("Failed to focus window: {}", e))?;
        return Ok(());
    }
    
    // Create new window
    let _ = WebviewWindowBuilder::new(
        &app_handle,
        &window_id,  // Unique identifier for this window
        WebviewUrl::App(page_path.into())  // Path relative to dist folder (e.g., "settings.html")
    )
    .title("Secondary Window")
    .inner_size(800.0, 600.0)
    .min_inner_size(400.0, 300.0)
    .resizable(true)
    .center(true)
    .decorations(true)  // Show title bar and window controls
    .transparent(false)
    .always_on_top(false)
    .build()
    .map_err(|e| format!("Failed to create window: {}", e))?;
    
    Ok(())
}
```

### 3. Register Command in main.rs
```rust
tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
        open_secondary_window,
        // ... other commands
    ])
    .run(tauri::generate_context!())
```

### 4. Call from Frontend
```javascript
import { invoke } from '@tauri-apps/api/core';

// Open a settings window
await invoke('open_secondary_window', {
    windowId: 'settings-window',
    pagePath: 'settings.html'
});
```

## Method 2: JavaScript Frontend

### 1. Frontend Implementation
```javascript
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';

async function openSecondaryWindow(windowId, pagePath, title = 'Secondary Window') {
    try {
        // Check if window already exists
        const existingWindows = await WebviewWindow.getAll();
        const existingWindow = existingWindows.find(w => w.label === windowId);
        
        if (existingWindow) {
            // Focus existing window
            await existingWindow.setFocus();
            return;
        }
        
        // Create new window
        const webview = new WebviewWindow(windowId, {
            url: `/${pagePath}`, // Path relative to dist folder
            title: title,
            width: 800,
            height: 600,
            minWidth: 400,
            minHeight: 300,
            resizable: true,
            center: true,
            decorations: true,
            transparent: false,
            alwaysOnTop: false
        });
        
        // Optional: Handle window events
        webview.once('tauri://created', () => {
            console.log('Window created successfully');
        });
        
        webview.once('tauri://error', (error) => {
            console.error('Window creation failed:', error);
        });
        
    } catch (error) {
        console.error('Failed to create window:', error);
    }
}
```

## Required Configuration

### 1. Content Security Policy (tauri.conf.json)
Ensure your CSP allows the necessary sources:

```json
{
  "app": {
    "security": {
      "csp": {
        "default-src": "'self' https: http: data: ws: wss: blob: file: customprotocol: asset:",
        "script-src": "'self' 'unsafe-eval' 'unsafe-inline'",
        "style-src": "'self' 'unsafe-inline'",
        "img-src": "'self' asset: data: blob: http: https:",
        "connect-src": "'self' ipc: http://ipc.localhost http: https: ws: wss:"
      }
    }
  }
}
```

### 2. Main Window Configuration
The main window should be defined in `tauri.conf.json`:

```json
{
  "app": {
    "windows": [
      {
        "title": "Main Application",
        "label": "main",
        "width": 800,
        "height": 1000,
        "minWidth": 720,
        "minHeight": 600,
        "resizable": true,
        "center": true
      }
    ]
  }
}
```

## Window Management Best Practices

### 1. Prevent Duplicate Windows
Always check if a window with the same label already exists before creating a new one.

### 2. Unique Window Labels
Use descriptive, unique labels for each window type:
- `"main"` - Main application window
- `"settings-panel"` - Settings configuration
- `"log-viewer"` - Log display window
- `"document-preview"` - Document preview

### 3. Window Communication
Windows can communicate via Tauri's event system:

```rust
// Emit event to specific window
app_handle.emit_to("window-label", "event-name", payload)?;

// Emit to all windows
app_handle.emit_all("event-name", payload)?;
```

```javascript
// Listen for events in frontend
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen('event-name', (event) => {
    console.log('Received:', event.payload);
});
```

### 4. Cleanup
Close windows properly when no longer needed:

```rust
if let Some(window) = app_handle.get_webview_window("window-label") {
    window.close()?;
}
```

## File Structure
Ensure secondary window HTML files are in your build output:

```
dist/
├── index.html          (main window)
├── settings.html       (settings window)
├── log-viewer.html     (log window)
└── assets/
    ├── css/
    └── js/
```

## Common Patterns

### Toggle Window (Open/Close)
```rust
#[tauri::command]
async fn toggle_window(app_handle: AppHandle, window_id: String, page_path: String) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window(&window_id) {
        window.close().map_err(|e| format!("Failed to close window: {}", e))?;
    } else {
        WebviewWindowBuilder::new(&app_handle, &window_id, WebviewUrl::App(page_path.into()))
            .title("Toggle Window")
            .inner_size(600.0, 400.0)
            .build()
            .map_err(|e| format!("Failed to create window: {}", e))?;
    }
    Ok(())
}
```

### Modal-Style Window
```rust
WebviewWindowBuilder::new(&app_handle, "modal-window", WebviewUrl::App("modal.html".into()))
    .title("Modal Dialog")
    .inner_size(400.0, 300.0)
    .resizable(false)
    .always_on_top(true)
    .center(true)
    .decorations(true)
    .build()?;
```

## Key Points for LLMs

1. **No Popup Blocking**: Tauri windows are native OS windows, not browser popups
2. **Rust Backend Preferred**: More reliable for programmatic window creation
3. **Unique Labels Required**: Each window must have a distinct identifier
4. **Check for Duplicates**: Always verify if window exists before creating
5. **Proper CSP**: Ensure Content Security Policy allows necessary resources
6. **File Paths**: Use paths relative to the `dist` folder for HTML pages
7. **Event Communication**: Use Tauri's event system for inter-window communication

## Error Handling

Always wrap window operations in proper error handling:

```rust
.build()
.map_err(|e| format!("Failed to create window: {}", e))?;
```

This ensures graceful failure and proper error messages for debugging.