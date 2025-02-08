
import { Editor } from '@tiptap/core'
import StarterKit from '@tiptap/starter-kit'
import DynamicTextNode from './extensions/DynamicTextNode'
import DynamicTextMark from './extensions/DynamicTextMark'
import { RichLogEntryNode } from './extensions/RichLogEntryNode'
import { SimpleLogEntryNode } from './extensions/SimpleLogEntryNode'
import { listen } from '@tauri-apps/api/event';

// import { Editor } from 'https://esm.sh/@tiptap/core'
// import StarterKit from 'https://esm.sh/@tiptap/starter-kit'
const { invoke } = window.__TAURI__.core;

let greetInputEl;
let greetMsgEl;

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  //greetMsgEl.textContent = await invoke("greet", { name: greetInputEl.value });
  invoke("greet", { name: greetInputEl.value }).then((res) => {
    greetMsgEl.textContent = res + '. And this is JS Frontend saying hello!';
    console.log(editor);
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
    const pos = editor.state.selection.from + 2
    console.log(pos)
    addLogEntry({  
      id: '1',
      timestamp: new Date().toISOString(),
      message: 'Application started Now what? Writing objects: 100% (11/11), 1.55 KiB | 1.55 MiB/s, done.',
      level: 'info'
    })
    // Set selection to after the inserted content
    editor.commands.setTextSelection(pos)    //editor.chain().focus().insertContent('Hello World from Rust Backend '+greetInputEl.value+'<').run()
    editor.chain()
      //First insert regular content
      .focus().insertContent('Hello World from Rust Backend ' + greetInputEl.value)
      //Then insert our dynamic node as a separate block
      // .insertContent({
      //   type: 'dynamicText',
      //   attrs: { 
      //     id: 'node1',
      //     textColor: 'white'
      //   },
      //   content: [{ type: 'text', text: 'First node' }]
      // })
      .run()

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
  let unlistenFn;
  let unlistenRichLogMsg;
  console.log('***** Setting up event listener...');
  try {
    unlistenFn = await listen('simple-log-message', (event) => {
      console.log('Received simple-log-message:', event);
      if (event.payload) {
        addSimpleLogEntry({
          id: Date.now(),
          timestamp: event.payload.timestamp,
          message: event.payload.message,
          level: 'info'
        });
      }
    });
    unlistenRichLogMsg = await listen('rich-log-message', (event) => {
      console.log('Received rich-log-message:', event);
      if (event.payload) {
        addRichLogEntry({
          id: Date.now(),
          timestamp: event.payload.timestamp,
          message: event.payload.message,
          level: 'info'
        });
      }
    });
  } catch (error) {
    console.error('Failed to setup event listener:', error);
  }

  // Cleanup when window unloads
  window.addEventListener('unload', () => {
    if (unlistenFn) {
      unlistenFn();
    }
    if (unlistenRichLogMsg) {
      unlistenRichLogMsg();
    }
  });
});

const editor = new Editor({
  element: document.querySelector('.element'),
  extensions: [
    StarterKit,
    DynamicTextNode,
    DynamicTextMark,
  ],
  // content: '<p>Hello World! This is the Editor</p>',
})

const diagnostics = new Editor({
  element: document.querySelector('.diagnostics'),
  extensions: [
    StarterKit,
    RichLogEntryNode,
    DynamicTextMark,
    SimpleLogEntryNode,
  ],
})

function addRichLogEntry(entry) {
  let pos = editor.state.selection.from + 2
  editor.commands.setTextSelection(pos)
  diagnostics.commands.insertContent({
    type: 'richLogEntry',
    attrs: {
      id: entry.id,
      timestamp: entry.timestamp,
      message: entry.message,
      level: entry.level,
    }
  })
  pos = diagnostics.state.selection.from + 2
  diagnostics.commands.setTextSelection(pos)
}

function addSimpleLogEntry(entry) {
  let pos = editor.state.selection.from + 2
  editor.commands.setTextSelection(pos)
  diagnostics.commands.insertContent({
    type: 'simpleLogEntry',
    attrs: {
      id: entry.id,
      timestamp: entry.timestamp,
      message: entry.message,
      level: entry.level,
    }
  })
  pos = diagnostics.state.selection.from + 1
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
    if (node.type.name === 'dynamicText' && node.attrs.id === id) {
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

function truncateText(text, maxLength = 100) {
  return text.length > maxLength 
      ? text.slice(0, maxLength) + '...' 
      : text;
}
