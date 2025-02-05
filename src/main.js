
import { Editor } from '@tiptap/core'
import StarterKit from '@tiptap/starter-kit'
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
    editor.chain().focus().insertContent('Hello World from Rust Backend '+greetInputEl.value+'<').run()
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
  extensions: [StarterKit],
  content: '<p>Hello World! This is the Editor</p>',
})
