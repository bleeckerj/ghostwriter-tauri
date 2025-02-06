
import { Editor } from '@tiptap/core'
import StarterKit from '@tiptap/starter-kit'
import DynamicTextNode from './extensions/DynamicTextNode'
console.log('DynamicTextNode:', DynamicTextNode)


// import { Editor } from 'https://esm.sh/@tiptap/core'
// import StarterKit from 'https://esm.sh/@tiptap/starter-kit'
const { invoke } = window.__TAURI__.core;

let greetInputEl;
let greetMsgEl;

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  //greetMsgEl.textContent = await invoke("greet", { name: greetInputEl.value });
  invoke("greet", { name: greetInputEl.value }).then((res) => {
    greetMsgEl.textContent = res;
    console.log(editor);
    //editor.chain().focus().insertContent('Hello World from Rust Backend '+greetInputEl.value+'<').run()
    editor.chain()
      .focus()
      // First insert regular content
      //.insertContent('Hello World from Rust Backend ' + greetInputEl.value)
      // Then insert our dynamic node as a separate block
      .insertContent({
        type: 'dynamicText',
        attrs: { 
          id: 'node1',
          textColor: 'white'
        },
        content: [{ type: 'text', text: 'First node ' }]
      })
      .run()

  });
}

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  document.querySelector("#greet-form").addEventListener("submit", (e) => {
    e.preventDefault();
    greet();
  });
});


const editor = new Editor({
  element: document.querySelector('.element'),
  extensions: [
    StarterKit,
    DynamicTextNode,
  ],
  content: '<p>Hello World! This is the Editor</p>',
})


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

// Example: Change color after 3 seconds
setTimeout(() => {
  updateNodeColor('123', '#ff0000')
}, 3000)