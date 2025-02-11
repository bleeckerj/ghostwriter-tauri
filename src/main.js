import { Editor } from '@tiptap/core'
import StarterKit from '@tiptap/starter-kit'
//import DynamicTextNode from './extensions/DynamicTextNode'
import DynamicTextMark from './extensions/DynamicTextMark'
//import DiagnosticLogEntryNode from './extensions/DiagnosticLogEntryNode'
import SimpleLogEntryNode from './extensions/SimpleLogEntryNode'
import { InlineActionItem } from './extensions/InlineActionItem'

import { listen } from '@tauri-apps/api/event';
import RichLogEntryNode from './extensions/RichLogEntryNode'
import { ProgressExtension } from './extensions/ProgressNode';
//import { Placeholder } from '@tiptap/extension-placeholder'
const { invoke } = window.__TAURI__.core;

let greetInputEl;
let greetMsgEl;
//let greetBtnEl;
let incantBtnEl;

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  //greetMsgEl.textContent = await invoke("greet", { name: greetInputEl.value });
  invoke("greet", { name: greetInputEl.value }).then(res => {
    greetMsgEl.textContent =  'And this is JS Frontend saying hello!';
    editor.chain()
    .focus()
    .insertContent([
      {
        type: 'text',
        text: ' '
      },
      {
        type: 'text',
        text: res,
        marks: [{
          type: 'dynamicTextMark',
          attrs: { 
            textColor: 'black',
            backgroundColor: '#f3f4f6',
            twMisc: 'font-regular font-[WarblerText] rounded animated-highlight',
            id: 'backend-id-123',
            timestamp: Date.now(),
            raw: res
          }  
        }]
      },
      {
        type: 'text',
        text: ' '
      }
    ]).run()
    
    const pos = editor.state.selection.from + 3
    console.log(pos)
    editor.commands.setTextSelection(pos)   
    
  });
}

async function searchSimilarity() {
  const results = await invoke("search_similarity", { 
    query: editor.getText(), 
    limit: 4 
  });
  
  // Add log entries for the results
  results.forEach((result, index) => {
    addSimpleLogEntry({ 
      id: Date.now() + "_" + index,
      timestamp: Date.now(),
      message: `<div>
        <p class='border-l-[4px] border-pink-100 pl-2 pr-8 text-pretty'>${result.chunk_text}</p>
        <p class='mt-1 px-2 py-1 rounded-sm bg-gray-800 w-fit'>${result.similarity_score}</p>
        <span class='font-bold'>${result.document_name}</span>
      </div>`,
      level: 'info'
    });
  });

  // Return the results for further use
  return results;
}

async function completionFromContext() {
  let dots = 0;
  const loadingInterval = setInterval(() => {
    dots = (dots + 1) % 4;
    greetMsgEl.textContent = `Emanating${'.'.repeat(dots)}`;
  }, 250);
  

  
  invoke("completion_from_context", { input: editor.getText() })
  .then(([content, timing]) => {
    clearInterval(loadingInterval);
    greetMsgEl.textContent = 'Complete';
    // // addSimpleLogEntry({
    // //   id: Date.now(),
    // //   timestamp: Date.now(),
    // //   message: `Completion timing: 
    // //     Embedding: ${timing.embedding_generation_ms}ms, 
    // //     Similarity: ${timing.similarity_search_ms}ms, 
    // //     Emanation: ${timing.openai_request_ms}ms, 
    // //     Total: ${timing.total_ms}ms`,
    // //   level: 'info'
    // // });
    // // console.log("Timing (ms):", {
    //   "Embedding": timing.embedding_generation_ms,
    //   "Similarity": timing.similarity_search_ms,
    //   "Emanation": timing.openai_request_ms,
    //   "Total": timing.total_ms
    // });
    console.log("Completion content:", content);
    editor.chain()
    .focus()
    .insertContent([
      // {
      //   type: 'text',
      //   text: ''
      // },
      {
        type: 'text',
        text: content,
        marks: [{
          type: 'dynamicTextMark',
          attrs: { 
            textColor: 'blue',
            backgroundColor: '#f3f4f6',
            twMisc: 'font-regular font-[WarblerText] text-[1em] rounded animated-highlight',
            id: 'backend-id-123',
            timestamp: Date.now(),
            raw: content
          }  
        }]
      },
      {
        type: 'text',
        text: ' '
      }
    ]).run();
  })
  .catch((err) => {
    clearInterval(loadingInterval);
    greetMsgEl.textContent = 'Error occurred';
    console.error(err);
  });
}

window.addEventListener("DOMContentLoaded", async () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  //greetBtnEl = document.querySelector("#greet-btn");
  //greetBtnEl.addEventListener("click", searchSimilarity);
 // greetBtnEl.addEventListener("click", greet);
  incantBtnEl = document.querySelector("#incant-btn");
  incantBtnEl.addEventListener("click", completionFromContext);
  // document.querySelector("#greet-form").addEventListener("submit", (e) => {
    //   e.preventDefault();
  //   greet();
  // });
  // Add Tauri event listener here
  let unlistenSimpleLogMessageFn;
  let unlistenRichLogMessageFn;
  let unlistenProgressIndicatorUpdateFn;
  let unlistenProgressIndicatoLoadrFn;
  try {
    unlistenSimpleLogMessageFn = await listen('simple-log-message', (event) => {
      console.log('Received event:', event);
      if (event.payload) {
        addSimpleLogEntry({
          id: event.payload.id,
          id: event.payload.id,
          timestamp: event.payload.timestamp,
          message: event.payload.message,
          level: event.payload.level
        });
      }
    });
  } catch (error) {
    console.error('Failed to setup event listener:', error);
  }
  
  try {
    unlistenRichLogMessageFn = await listen('rich-log-message', (event) => {
      console.log('Received event:', event);
      if (event.payload) {
        addRichLogEntry({
          id: Date.now(),
          timestamp: event.payload.timestamp,
          message: event.payload.message,
          data: event.payload.data,
          level: 'warn'
        });
      }
    });
  } catch (error) {
    console.error('Failed to setup event listener:', error);
  }
  
  try {
    unlistenProgressIndicatoLoadrFn = await listen('load-progress-indicator', (event) => {
      console.log('Progress Indicator Received event:', event);
      if (event.payload) {
        addProgressIndicatorNode({
          progress_id: event.payload.progress_id,
          current_step: event.current_step,
          total_steps: event.total_steps,
          current_file: event.payload.current_file,
          meta: event.payload.meta
        });
      }
    });
  } catch (error) {
    console.error('Failed to setup event listener:', error);
  }
  
  try {
    unlistenProgressIndicatorUpdateFn = await listen('progress-indicator-update', (event) => {
      console.log('Received event:', event);
      if (event.payload) {
        window.updateProgressNode(diagnostics, event.payload.progress_id, {
          current_step: event.payload.current_step,
          current_file: event.payload.current_file,
          total_steps: event.payload.total_steps,
          meta: event.payload.meta
        })
      }
      if (event.payload && event.payload.current_step === event.payload.total_steps) {
        window.updateProgressNode(diagnostics, event.payload.progress_id, {
          current_step: event.payload.current_step,
          current_file: event.payload.current_file,
          total_steps: event.payload.total_steps,
          meta: "Completed Ingestion"
        })
      }
    });
  } catch (error) {
    console.error('Failed to setup event listener:', error);
  }
  
  invoke("simple_log_message", { message: 'Ghostwriter Up.', id: "tracker", level: "info" }).then((res) => {
    console.log('simple_log_emissions', res);
  });
  // invoke("rich_log_message", { message: 'Ghostwriter Up.', data: "no data", level: "info" }).then((res) => {
  //   console.log('rich_log_emissions', res);
  // });
  // Cleanup when window unloads
  window.addEventListener('unload', () => {
    if (unlistenFn) {
      unlistenFn();
    }
  });
  //   addSimpleLogEntry({
  //     id: '1',
  //     timestamp: new Date().toISOString(),
  //     message: 'Ghostwriter Up.',
  //     level: 'info'
  //   }).run()
  
  // Initialize the resize handle
  initializeResizeHandle();
});

const editor = new Editor({
  element: document.querySelector('.element'),
  extensions: [
    StarterKit,
    //DynamicTextNode,
    DynamicTextMark,
    InlineActionItem,
    //SimpleLogEntryNode,
    //DiagnosticLogEntryNode
  ],
  // content: '<p>Hello World! This is the Editor</p>',
})

const diagnostics = new Editor({
  element: document.querySelector('.diagnostics'),
  extensions: [
    StarterKit,
    //DiagnosticLogEntryNode,
    SimpleLogEntryNode,
    RichLogEntryNode,
    DynamicTextMark,
    ProgressExtension  
  ],
})

function addSimpleLogEntry(entry) {
  let pos = diagnostics.state.selection.from + 2
  diagnostics.commands.setTextSelection(pos)
  diagnostics.commands.insertContent({
    type: 'simpleLogEntry',
    attrs: {
      id: entry.id,
      timestamp: entry.timestamp,
      message: entry.message,
      level: entry.level,
    }
  })
  pos = diagnostics.state.selection.from + 2
  diagnostics.commands.setTextSelection(pos)
  setTimeout(() => {
    diagnostics.view.dom.scrollIntoView({ behavior: 'smooth', block: 'end' });
  }, 0);
}

function addProgressIndicatorNode(entry) {
  let pos = diagnostics.state.selection.from + 2
  diagnostics.commands.setTextSelection(pos)
  diagnostics.commands.insertContent({
    type: 'progressIndicator',
    attrs: {
      progress_id: entry.progress_id,
      current_step: entry.current_step,
      total_steps: entry.total_steps,
      current_file: entry.current_file
    }
  })
  pos = diagnostics.state.selection.from + 2
  diagnostics.commands.setTextSelection(pos)
  setTimeout(() => {
    diagnostics.view.dom.scrollIntoView({ behavior: 'smooth', block: 'end' });
  }, 0);
}

function addRichLogEntry(entry) {
  let pos = diagnostics.state.selection.from + 2
  diagnostics.commands.setTextSelection(pos)
  diagnostics.commands.insertContent({
    type: 'richLogEntry',
    attrs: {
      id: entry.id,
      timestamp: entry.timestamp,
      message: entry.message,
      data: entry.data,
      level: entry.level,
    }
  })
  pos = diagnostics.state.selection.from + 2
  diagnostics.commands.setTextSelection(pos)
  setTimeout(() => {
    diagnostics.view.dom.scrollIntoView({ behavior: 'smooth', block: 'end' });
  }, 0);
}



// Function to update node color
function updateNodeColor(id, newColor) {
  const { state, view } = editor
  const { tr } = state
  let hasUpdated = false
  
  state.doc.descendants((node, pos) => {
    if (node.type.name === 'dynamicTextMark' && node.attrs.id === id) {
      tr.setNodeMarkup(pos, null, {
        ...node.attrs,
        textColor: newColor
      })
      hasUpdated = true
    }
  })
  
  if (hasUpdated) {
    view.dispatch(tr)
  }
}

function initializeResizeHandle() {
  const handle = document.querySelector('.resize-handle');
  const topArea = document.querySelector('.scroll-area');
  const bottomArea = document.querySelector('.diagnostics-scroll-area');
  let startY;
  let startHeights;
  const MIN_HEIGHT_BOTTOM = 16; // 1rem = 16px typically
  const MIN_HEIGHT_TOP = 48; // 3rem = 48px typically
  
  function startResize(e) {
    startY = e.clientY;
    startHeights = {
      top: topArea.offsetHeight,
      bottom: bottomArea.offsetHeight
    };
    
    document.addEventListener('mousemove', resize);
    document.addEventListener('mouseup', stopResize);
    document.body.style.cursor = 'row-resize';
  }
  
  function resize(e) {
    const delta = e.clientY - startY;
    const containerHeight = topArea.parentElement.offsetHeight;
    const newTopHeight = startHeights.top + delta;
    const newBottomHeight = startHeights.bottom - delta;
    
    // Check if bottom area should collapse (dragging down)
    if (newBottomHeight < MIN_HEIGHT_BOTTOM) {
      bottomArea.classList.add('collapsed');
      topArea.style.flex = '1';
      bottomArea.style.flex = '0';
      return;
    }
    
    // Check if top area would become too small (dragging up)
    if (newTopHeight < MIN_HEIGHT_TOP) {
      return;
    }
    
    // Only remove collapsed class if explicitly uncollapsing
    if (bottomArea.classList.contains('collapsed') && delta < 0) {  // Only when dragging up
      bottomArea.classList.remove('collapsed');
      bottomArea.style.flex = '1';
    }
    
    // Only update flex values if not collapsed
    if (!bottomArea.classList.contains('collapsed')) {
      // Use containerHeight directly since we don't need to subtract MIN_HEIGHT anymore
      const topPercent = (newTopHeight / containerHeight) * 100;
      const bottomPercent = (newBottomHeight / containerHeight) * 100;
      
      topArea.style.flex = `${topPercent} 1 0`;
      bottomArea.style.flex = `${bottomPercent} 1 0`;
    }
  }
  
  function stopResize() {
    document.removeEventListener('mousemove', resize);
    document.removeEventListener('mouseup', stopResize);
    document.body.style.cursor = '';
  }
  
  // Double-click handler to toggle collapse
  handle.addEventListener('dblclick', () => {
    if (bottomArea.classList.contains('collapsed')) {
      bottomArea.classList.remove('collapsed');
      topArea.style.flex = '2';
      bottomArea.style.flex = '1';
    } else {
      bottomArea.classList.add('collapsed');
      topArea.style.flex = '1';
    }
  });
  
  handle.addEventListener('mousedown', startResize);
}