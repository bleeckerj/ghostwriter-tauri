import { Editor } from '@tiptap/core'
import StarterKit from '@tiptap/starter-kit'
//import DynamicTextNode from './extensions/DynamicTextNode'
import DynamicTextMark from './extensions/DynamicTextMark'
//import DiagnosticLogEntryNode from './extensions/DiagnosticLogEntryNode'
import SimpleLogEntryNode from './extensions/SimpleLogEntryNode'
import { listen } from '@tauri-apps/api/event';
import RichLogEntryNode from './extensions/RichLogEntryNode'
import { ProgressExtension } from './extensions/ProgressNode';
//import { Placeholder } from '@tiptap/extension-placeholder'
import { InlineActionItem } from './extensions/InlineActionItem';
import { PluginKey } from 'prosemirror-state';
//import {Menu, Submenu} from '@tauri-apps/api/menu'

import { open } from '@tauri-apps/plugin-dialog';

const { invoke } = window.__TAURI__.core;

let greetInputEl;
let greetMsgEl;
//let greetBtnEl;
let incantBtnEl;

// const macOS = navigator.userAgent.includes('Macintosh')

// async function createMenuWithSubmenu() {
//   const submenu = await Submenu.new({
//     text: 'Options',
//     items: [
//       {
//         id: 'option1',
//         text: 'Option 1',
//         action: () => { console.log('Option 1 clicked'); }
//       },
//       {
//         id: "option2",
//         text: "Option 2",
//         action: () => { console.log("Option 2 clicked"); },
//       },
//     ],
//   });

//   const menu = await Menu.new({ items: [submenu] });

//   menu.setAsAppMenu();
// }

//createMenuWithSubmenu();

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  //greetMsgEl.textContent = await invoke("greet", { name: greetInputEl.value });
  invoke("greet", { name: greetInputEl.value }).then(res => {
    greetMsgEl.textContent =  'And this is JS Frontend saying hello!';
    //openDialogTest();
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
            twMisc: 'rounded animated-highlight',
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

/** need to handle this asynchronously and the menu handler in Rust is synchronous
* so we have to have Rust tell the frontend to open the dialog
* and then we get the file path and send it back to Rust for ingestion
*/
async function openDialogForIngestion() {
  // Open a dialog
  const file = await open({
    multiple: false,
    directory: false,
  });
  //console.log(file);
  const foo = await invoke("ingestion_from_file_dialog", {
    filePath: file
  }).then((res) => {
    console.log(res);
    return res;
  }
);
console.log("ingestion result ", foo);
// const results = await invoke("search_similarity", {
//   query: file,
//   limit: 4
// });
// console.log(results);
// // Prints file path or URI
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
            twMisc: 'rounded animated-highlight',
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
  //create();
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  //greetBtnEl = document.querySelector("#greet-btn");
  //greetBtnEl.addEventListener("click", searchSimilarity);
  // greetBtnEl.addEventListener("click", greet);
  incantBtnEl = document.querySelector("#incant-btn");
  incantBtnEl.textContent = "INGEST";
  incantBtnEl.addEventListener("click", openDialogForIngestion);
  // document.querySelector("#greet-form").addEventListener("submit", (e) => {
    //   e.preventDefault();
  //   greet();
  // });
  // Add Tauri event listener here
  let unlistenSimpleLogMessageFn;
  let unlistenRichLogMessageFn;
  let unlistenProgressIndicatorUpdateFn;
  let unlistenProgressIndicatorLoadFn;
  let unlistenOpenFileDialogForIngestFn;
  let unlistenCanonListFn;
  try {
    unlistenSimpleLogMessageFn = await listen('simple-log-message', (event) => {
      console.log('Received event:', event);
      if (event.payload) {
        addSimpleLogEntry({
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
    unlistenCanonListFn = await listen('canon-list', (event) => {
      console.log('This is the event:', event);
      console.log('Payload:', event.payload);
  
      try {
        const listing = JSON.parse(event.payload); // Parse the JSON string
        console.log('Parsed listing:', listing);
  
        listing.documents.forEach((doc, index) => {
          console.log(`Document ${index}:`, doc);
          // Access document properties:
          // console.log(`  ID: ${doc.id}`);
          // console.log(`  Name: ${doc.name}`);
          // console.log(`  File Path: ${doc.file_path}`);
          // console.log(`  Created At: ${doc.created_at}`);
  
          // You can now use the 'doc' object to create a rich log entry, for example:
          addRichLogEntry({
            id: ""+doc.id,
            timestamp: doc.created_at,
            message: doc.name,
            data: doc.id, // Or any other data you want to include
            level: 'info',
          });
        });
      } catch (error) {
        console.error('Error parsing or iterating over payload:', error);
      }
    });
  } catch (error) {
    console.error('Failed to setup event listener:', error);
  }
  

  try {
    unlistenOpenFileDialogForIngestFn = await listen('open-file-dialog-for-ingest', (event) => {
      console.log('Received event:', event);
      openDialogForIngestion();
    });  
  } catch (error) {
    console.error('Failed to setup event listener:', error);
  }
  
  try {
    unlistenOpenFileDialogForIngestFn = await listen('open-canon-list', (event) => {
      console.log('Hey Received event:', event);
    }
  );  
} catch (error) {
  console.error('Failed to setup event listener:', error);
}

try {
  unlistenRichLogMessageFn = await listen('rich-log-message', (event) => {
    console.log('Received rich-log-message event:', event);
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
  unlistenProgressIndicatorLoadFn = await listen('progress-indicator-load', (event) => {
    console.log('Progress Indicator Received Load Event:', event);
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
    console.log('Progress Indicator Received Update Event:', event);
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
  if (unlistenSimpleLogMessageFn) {
    unlistenSimpleLogMessageFn();
  }
  if (unlistenRichLogMessageFn) {
    unlistenRichLogMessageFn();
  }
  if (unlistenProgressIndicatorUpdateFn) {
    unlistenProgressIndicatorUpdateFn();
  }
  if (unlistenProgressIndicatorLoadFn) {
    unlistenProgressIndicatorLoadFn();
  }
  if (unlistenOpenFileDialogForIngestFn) {
    unlistenOpenFileDialogForIngestFn();
  }
  if (unlistenCanonListFn) {
    unlistenCanonListFn();
  }
});

// Initialize the resize handle
initializeResizeHandle();
});

const editor = new Editor({
  element: document.querySelector('.element'),
  extensions: [
    StarterKit,
    DynamicTextMark,
    InlineActionItem.configure({
      disabled: false,                // Disables the feature
      timeout: 3000,                 // Show button after 3 seconds
      onClick: async (view, pos, event) => {
        try {
          // Show loading state in the message element
          greetMsgEl.textContent = 'Processing...';
          
          // Call your completion function
          await completionFromContext();
          
          // Re-enable the plugin after completion
          // const pluginKey = new PluginKey('inlineActionItem');
          // const tr = view.state.tr.setMeta(pluginKey, { disabled: false });
          // view.dispatch(tr);
          
          // Update message
          greetMsgEl.textContent = 'Completed';
        } catch (error) {
          console.error('Action failed:', error);
          greetMsgEl.textContent = 'Error occurred';
          
          // Make sure to re-enable even on error
          const pluginKey = new PluginKey('inlineActionItem');
          const tr = view.state.tr.setMeta(pluginKey, { disabled: false });
          view.dispatch(tr);
        }
      },
    }),
  ],
})

const diagnostics = new Editor({
  element: document.querySelector('.diagnostics'),
  extensions: [
    StarterKit,
    //DiagnosticLogEntryNode,
    SimpleLogEntryNode,
    RichLogEntryNode.configure({
      onDelete: ({ node, getPos, editor }) => {
        const pos = getPos(); // Get the position immediately
        const doc_id = node.attrs.id; // Get the ID of the node which should be the doc_id
        invoke("delete_canon_entry", { docid: doc_id }).then((res) => {

        });
        // Check if the position is valid and the node is still in the document
        if (typeof pos === 'number' && pos >= 0 && pos < editor.state.doc.content.size) {
          // The position is valid, so proceed with deleting the node
          editor.chain().focus().deleteRange({ from: pos, to: pos + 1 }).run();
        } else {
          // The position is invalid, so log an error and do nothing
          console.error('Invalid position for node:', node, pos);
        }
      },
    }),
    DynamicTextMark,
    ProgressExtension,
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