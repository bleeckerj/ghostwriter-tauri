import { Editor } from '@tiptap/core'
import StarterKit from '@tiptap/starter-kit'
//import DynamicTextNode from './extensions/DynamicTextNode'
import DynamicTextMark from './extensions/DynamicTextMark'
//import DiagnosticLogEntryNode from './extensions/DiagnosticLogEntryNode'
import SimpleLogEntryNode from './extensions/SimpleLogEntryNode'
import { listen } from '@tauri-apps/api/event';
import RichLogEntryNode from './extensions/RichLogEntryNode'
import CanonEntryNode from './extensions/CanonEntryNode'
import { LogicalPosition, PhysicalPosition } from '@tauri-apps/api/dpi';  // Add this import

import { ProgressExtension } from './extensions/ProgressNode';
//import { Placeholder } from '@tiptap/extension-placeholder'
import { InlineActionItem } from './extensions/InlineActionItem';
import { PluginKey } from 'prosemirror-state';
//import {Menu, Submenu} from '@tauri-apps/api/menu'

import { open, save, confirm } from '@tauri-apps/plugin-dialog';
import { getCurrentWebviewWindow, WebviewWindow } from '@tauri-apps/api/webviewWindow';
//import { list } from 'postcss';
import { Timer } from './timer.js';
import { debounce, set } from 'lodash';

let unlistenCanonListFn;

const canon_list = new Editor({
    element: document.querySelector('.canon-editor'),
    editable: false,
    extensions: [
        StarterKit,
        //DiagnosticLogEntryNode,
        SimpleLogEntryNode,
        RichLogEntryNode.configure({
            //onDelete: handleRichLogEntryDelete,
        }),
        CanonEntryNode.configure({
            onDelete: ({ node, getPos, editor }) => {
                // Your existing delete handling code
                console.log('Deleting canon with ID:', node.attrs.id)
                editor.commands.deleteNode('canonEntry')
                //handleCanonEntryDelete({ node, getPos, editor })
            },
            // Add this new handler
            onTogglePause: ({ node, isPaused }) => {
                console.log('Toggle pause state:', isPaused, 'for node ID:', node.attrs.id)
                
                // Add any additional logic needed when pause state changes
                // For example, if you need to call a Tauri function:
                // invoke('toggle_rag_pause', { id: node.attrs.id, paused: isPaused })
                invoke('toggle_rag_pause', { id: node.attrs.id, paused: isPaused })
                .then((res) => {
                    console.log('Toggle pause state:', isPaused, 'for canon with ID:', node.attrs.id, 'Response:', res)
                })
                .catch((error) => {
                    console.error('Failed to toggle pause state:', error)
                });
            }
        }),
        DynamicTextMark,
        ProgressExtension,
    ],
})

function addCanonEntry(entry) {
    let pos = canon_list.state.selection.from + 2
    canon_list.commands.setTextSelection(pos)
    canon_list.commands.insertContent({
        type: 'canonEntry',
        attrs: {
            id: entry.id,
            timestamp: entry.timestamp,
            message: entry.message,
            data: entry.data,
            embedding_model_name: entry.embedding_model_name,
            notes: entry.notes,
            paused: entry.paused,
            level: entry.level,
        }
    })
    console.log('Added canon entry:', entry);
    pos = canon_list.state.selection.from + 2
    canon_list.commands.setTextSelection(pos)
    setTimeout(() => {
        canon_list.view.dom.scrollIntoView({ behavior: 'smooth', block: 'end' });
    }, 0);
}


try {
    unlistenCanonListFn = await listen('canon-list', (event) => {
        // console.log('This is the event:', event);
        // console.log('Payload:', event.payload);
        
        try {
            const listing = JSON.parse(event.payload); // Parse the JSON string
            console.log('Parsed listing:', listing);
            
            listing.documents.forEach((doc, index) => {
                // console.log(`Document ${index}:`, doc);
                // console.log(`  ID: ${doc.id}`);
                // console.log(`  Name: ${doc.name}`);
                // console.log(`  File Path: ${doc.file_path}`);
                // console.log(`  Created At: ${doc.created_at}`);
                // console.log(`  Embedding Model Name: ${doc.embedding_model_name}`);
                // console.log(`  Notes: ${doc.notes}`);
                
                // You can now use the 'doc' object to create a rich log entry, for example:
                addCanonEntry({
                    id: ""+doc.id,
                    timestamp: doc.created_at,
                    message: doc.name,
                    data: doc.id, 
                    embedding_model_name: doc.embedding_model_name,
                    notes: doc.notes,
                    paused: doc.paused,// Or any other data you want to include
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

window.addEventListener('load', () => {
    console.log('Window loaded, invoking showCanonList');
    showCanonList();
    // const timer = new Timer(1000, () => {
    //     console.log('Timer tick');
    //     showCanonList();
    // });
    // timer.start();
});

window.addEventListener('unload', () => {
    if (unlistenCanonListFn) {
        unlistenCanonListFn();
    }
});

async function showCanonList() {
    try {
      await invoke("list_canon_docs", { limit: 99 });
      //let pluginKey = new PluginKey('inlineActionItem');
      //console.log('Plugin Key:', pluginKey);
      // const tr = view.state.tr.setMeta(pluginKey, { disabled: true });
      // view.dispatch(tr);
    } catch (error) {
      console.error('Failed to list canon docs:', error);
      addSimpleLogEntry({
        id: Date.now(),
        timestamp: Date.now(),
        message: 'Failed to list canon docs from backend: '+error,
        level: 'error'
      });
    }
  }
