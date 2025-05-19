# ✍️ Ghostwriter™

> _Ghostwriter writes with you… not for you._

Ghostwriter is a locally-run, AI-inflected writing companion for creative thinkers, speculative designers, and anyone who writes to discover what they think. It doesn’t just autocomplete — it remembers your writing, retrieves your voice, and responds like a future version of yourself.

---

## 🔮 What is Ghostwriter?

Ghostwriter is:

- 🧠 A **semantic memory engine** that embeds and indexes your writing corpus
- ✨ A **context-aware autocompleter** powered by GPT-4, Ollama, or LM Studio
- 💻 A **terminal-based co-writer** with a beautiful TUI interface
- 🌀 A **creative mode** for co-authoring with AI in real-time called **Vibewriter™**

---

## 🌟 What Users Say

> “Ghostwriter doesn’t just finish my sentences — it finishes my thoughts.”  
> — *Wm. Branson, Journalist & Novelist*

> “It’s like having a future version of me in the loop.”  
> — *Larry Greenberg, Writing Coach*

> “Ghostwriter has become part of my writing ritual. I can’t imagine drafting without it.”  
> — *Chaya Trent, Tech Strategist*

    <svg width="16" height="16" viewBox="0 0 24 24" fill="#fbbf24" xmlns="http://www.w3.org/2000/svg">
      <path d="M12 17.27L18.18 21 16.54 13.97 22 9.24l-7.19-.61L12 2 9.19 8.63 2 9.24 7.45 13.97 5.82 21z"/>
    </svg>

---

## 🌟 Key Features

### 📚 Personal Document Memory
Ghostwriter ingests `.txt`, `.md`, and `.pdf` files and embeds them with `text-embedding-ada-002` (or your model). It stores them in a local SQLite vector store and recalls them using cosine similarity to enrich completions.

### 🔍 Contextual Autocomplete
When you type a sentence fragment, Ghostwriter finds semantically similar excerpts from your archive and builds an LLM prompt with:
- Recent conversation context
- Matching documents from your own corpus
- Your current writing fragment

Then it completes your thought — in your own voice.

### 🧠 Model Agnostic
Choose your backend:
- ✅ **OpenAI** (GPT-4, GPT-3.5)
- ✅ **Ollama** (run open-source models locally)
- ✅ **LM Studio** (point to a local inference server)

> *Ghostwriter doesn’t care who’s talking — as long as it remembers who you are.



### ⏱️ Vibewriter™ Mode
Vibewriter is a timed, improvisational writing mode where you and the Ghost take turns writing under a countdown. It's like tossing a medicine ball or rolling tractor tires with your AI muse..a high-intensity morning workout for your writer's brain.

- ⏳ **Timed Turns**: Set a tempo (10s, 30s, 60s, etc.)
- 👻 **Co-Authoring**: You write, the Ghost replies. Back and forth.
- 🎷 **Like Jazz**: It's not about control. “Trade fours” with the AI in speed rounds, learning to keep the flow going!
- 🌀 **A Creative Writing Workout - for your brain!**

🎯 Use it to: break creative ruts, riff toward new ideas, warm up for writing, or just have fun.

“Vibewriter feels like jamming with my muse, the source, not wrestling with my own brain.” — Beta tester


> _“Vibewriter feels like jamming with a ghost version of my own brain.”_ — Beta tester

### 🧾 Completion History & Logging
Every prompt, vector match, and AI response is saved as a `.jsonl` log entry so you can:
- Audit and reflect on your writing flow
- Fine-tune models later
- Export your sessions

🚀 What’s New in v1.3.0 “Luminous Echo”
    ✍️ Vibewriter Mode
    🧠 Model-Agnostic Inference Engine
    🗂️ Multi-format Ingestion (TXT, MD, PDF, URL)
    📜 Logging (JSONL)
    🔍 Smart Chunking + Semantic Recall
    💳 Corpus 2.0! Check in and out reference materials in real time!

---

🤖 How It Works
- Ingest: Drop .txt, .md, or .pdf files into the docs/ folder
- Chunk: Text is divided into overlapping chunks
- Embed: Each chunk gets a vector representation
- Store: Chunks + embeddings saved in SQLite DB
- Search: Input is embedded and matched to similar chunks
- Prompt: LLM gets context-rich prompt to generate completion
- Complete: You write — Ghostwriter listens and responds

---

## 🧭 Coming Soon

- **Visual Memory Overlay** — see what Ghostwriter sees when it completes.
- **Persona Cartridges** — swap in distinct voices, tonalities, and attitudes.
- **Multi-agent Collaboration** — have Ghostwriter integrate with Conversseract, and form an Pilshaw compliant Oraculator network — all in one interface.

---
# Ghostwriter Tauri Tiptap Vanilla Javascript

`2025-02-04T22:50:52-08:00`
Seems to build. Communicates from front to back to front again.
Lots of fiddling to get it to work.
Consists of Vite as well.
Moved Tauri backend from the `Ghostwriter Tauri Sandbox` repo and after much fiddling got it to build — but haven't yet run everything through.



# Tauri + Vanilla

This template should help get you started developing with Tauri in vanilla HTML, CSS and Javascript.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
