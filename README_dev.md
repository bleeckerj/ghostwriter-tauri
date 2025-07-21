# 🛠️ Ghostwriter Tauri App — Developer Guide

This document provides an overview of the architecture, components, and development workflow for the Ghostwriter Tauri application. It covers both the frontend and backend, describes how to run the app, and explains the functionality of the main UI buttons as defined in `index.html`, `main.js`, `lib.rs`, and `simplify.js`.

---

## 📦 Project Structure

```
ghostwriter-tauri/
├── src/                # Frontend (JS, CSS, HTML)
│   ├── main.js         # Main frontend logic
│   ├── simplify.js     # Simplify feature logic
│   ├── index.html      # Main UI layout and buttons
│   └── styles.css      # Styles
├── src-tauri/          # Tauri backend (Rust)
│   └── src/
│       └── lib.rs      # Main Rust backend logic and Tauri commands
├── package.json        # JS dependencies and scripts
├── tauri.conf.json     # Tauri configuration
└── ...
```

---

## 🚀 Getting Started

### Prerequisites

- **Node.js** (v18+ recommended)
- **npm** or **yarn**
- **Rust** (stable toolchain)
- **Tauri CLI**:  
  ```sh
  cargo install tauri-cli
  ```

### Setup & Run

1. **Install JS dependencies:**
   ```sh
   npm install
   # or
   yarn
   ```

2. **Run the app in development mode:**
   ```sh
   npm run tauri dev
   # or
   yarn tauri dev
   ```

   This will start both the frontend (Vite) and the Tauri backend.

---

## 🖼️ Frontend Overview

### Main UI (`index.html`)

The UI is composed of several action buttons and panels. The main action buttons are:

- **INGST** (`ingest-btn`): Ingests a document or URL into the semantic memory.
- **SSRCH** (`similarity-search-btn`): (Hidden by default) Triggers a similarity search.
- **NUDGE** (`nudge-inline-action-item`): (Hidden by default) Nudges the AI to provide a suggestion.
- **STRM** (`streaming-no-rag-mode-btn`): Starts streaming mode for AI completions. Click again to have a NO RAG streaming completion, which will be faster until like..vector search is super tight 🐎
- **SMLFY** (`simplify-btn`): Simplifies selected or all text using the LLM (see below).
- **PREFS** (`panel-toggle-btn`): Opens the preferences panel.
- **CLEAR** (`clear-diagnostics-btn`): Clears diagnostics/logs.

Other UI elements include the main editor, diagnostics area, and side panels.

---

## ⚙️ Frontend Logic

### `main.js`

- **Editor Initialization:** Sets up the Tiptap editor and connects it to the UI.
- **Button Event Handlers:** Wires up the action buttons to their respective features.
- **LLM Integration:** Uses Tauri's JS API to invoke backend commands for completions, ingestion, and simplification.
- **Simplify Feature:** Imports and calls `initSimplify(editor)` from `simplify.js` to enable the SMLFY button and related keyboard shortcuts.

### `simplify.js`

- **initSimplify(editor):**  
  - Enables the SMLFY button.
  - On click, sends either the selected text or the entire editor content to the backend for simplification.
  - Receives multiple simplified alternatives from the backend.
  - Allows the user to cycle through alternatives with Option+Up/Down arrows.
  - Escape reverts to the original text.
  - Ensures each line of the alternative appears as a separate paragraph in the editor.

---

## 🦀 Backend (Rust, Tauri)

### `lib.rs`

- **Tauri Commands:**  
  - `simplify_text`: Receives text, grade level, and number of alternatives. Constructs a detailed prompt (including social media engagement instructions), sends it to the selected LLM provider, and returns multiple alternatives.
  - Other commands handle ingestion, completions, and preferences.
- **LLM Provider Abstraction:** Supports OpenAI, Ollama, and LM Studio.
- **Prompt Engineering:** The simplify prompt is crafted to produce engaging, platform-appropriate, and readable alternatives.

---

## 🖱️ Button Functionality Reference

| Button ID                | Label   | Functionality                                                                                   |
|--------------------------|---------|-----------------------------------------------------------------------------------------------|
| `ingest-btn`             | INGST   | Ingests a document or URL into the semantic memory.                                            |
| `similarity-search-btn`  | SSRCH   | (Hidden) Triggers a similarity search in the corpus.                                           |
| `nudge-inline-action-item`| NUDGE  | (Hidden) Nudges the AI for a suggestion.                                                       |
| `streaming-no-rag-mode-btn`| SPEAK | Starts streaming mode for AI completions.                                                      |
| `simplify-btn`           | SMLFY   | Sends selected or all text to the backend for simplification and enhancement.                  |
| `panel-toggle-btn`       | PREFS   | Opens the preferences panel.                                                                   |
| `clear-diagnostics-btn`  | CLEAR   | Clears the diagnostics/log area.                                                               |

---

## 🧩 Extending & Debugging

- **Add new features:**  
  - Frontend: Add new buttons to `index.html`, wire up handlers in `main.js`.
  - Backend: Add new Tauri commands in `lib.rs` and register them in the handler.
- **Debugging:**  
  - Use browser dev tools for JS errors.
  - Use `console.log` or `addSimpleLogEntry` for logging in JS.
  - Use `log_message!` macro for backend logs.

---

## 📝 Notes

- The simplify feature is designed to work with both selected text and the entire editor content.
- Alternatives are formatted so each line appears as a separate paragraph in the editor.
- The backend prompt for simplification is highly tuned for social media and blog engagement.

---

For further questions, see the code comments in `main.js`, `simplify.js`, and `lib.rs