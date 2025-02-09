
import { Editor } from '@tiptap/core'
import StarterKit from '@tiptap/starter-kit'
//import DynamicTextNode from './extensions/DynamicTextNode'
import DynamicTextMark from './extensions/DynamicTextMark'
//import DiagnosticLogEntryNode from './extensions/DiagnosticLogEntryNode'
import SimpleLogEntryNode from './extensions/SimpleLogEntryNode'
import { listen } from '@tauri-apps/api/event';
import RichLogEntryNode from './extensions/RichLogEntryNode'
import { ProgressExtension } from './extensions/ProgressNode';
// import { Editor } from 'https://esm.sh/@tiptap/core'
// import StarterKit from 'https://esm.sh/@tiptap/starter-kit'
const { invoke } = window.__TAURI__.core;

let greetInputEl;
let greetMsgEl;

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  //greetMsgEl.textContent = await invoke("greet", { name: greetInputEl.value });
  invoke("greet", { name: greetInputEl.value }).then((res) => {
    greetMsgEl.textContent =  'And this is JS Frontend saying hello!';
    console.log(res);
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
          textColor: 'blue',
          backgroundColor: '#f3f4f6',
          twMisc: 'font-semibold rounded animated-highlight',
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
    ])
    .run()
    
    const pos = editor.state.selection.from + 3
    console.log(pos)
    editor.commands.setTextSelection(pos)   
    
    
    //editor.chain().focus().insertContent('Hello World from Rust Backend '+greetInputEl.value+'<').run()
    // editor.chain()
    // .focus()
    // .insertContent('Hello World from Rust Backend ' + greetInputEl.value)
    // .run()

    // invoke("rich_log_message", { message: 'Hello from JS Frontend!', level: "warn" }).then((res) => {
    //   console.log('test_log_emissions:', res);
    // });

    // addSimpleLogEntry({  
    //   id: '1',
    //   timestamp: new Date().toISOString(),
    //   message: 'Application started Now what? Writing objects: 100% (11/11), 1.55 KiB | 1.55 MiB/s, done.',
    //   level: 'info'
    // }).run()
    // Set selection to after the inserted content
   

  });
}

window.addEventListener("DOMContentLoaded", async () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  document.querySelector("#greet-form").addEventListener("submit", (e) => {
    e.preventDefault();
    greet();
  });
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
          progressId: event.payload.progressId,
          currentStep: event.currentStep,
          totalSteps: event.totalSteps,
          currentFile: event.payload.currentFile
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
        window.updateProgressNode(editor, event.payload.progressId, {
          currentStep: event.payload.currentStep,
          currentFile: event.payload.currentFile
        })
      }
    });
  } catch (error) {
    console.error('Failed to setup event listener:', error);
  }

  invoke("simple_log_message", { message: 'Ghostwriter Up.', id: "tracker", level: "info" }).then((res) => {
    console.log('simple_log_emissions', res);
  });
  invoke("rich_log_message", { message: 'Ghostwriter Up.', data: "no data", level: "info" }).then((res) => {
    console.log('rich_log_emissions', res);
  });
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

});

const editor = new Editor({
  element: document.querySelector('.element'),
  extensions: [
    StarterKit,
    //DynamicTextNode,
    DynamicTextMark,
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
      progressId: entry.progressId,
      currentStep: 0,
      totalSteps: 300,
      currentFile: 'document.pdf'
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