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

import { open, confirm, prefsSaveBtn } from '@tauri-apps/plugin-dialog';
import { getCurrentWebviewWindow, WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { list } from 'postcss';


let w = getCurrentWebviewWindow();
w.setBackgroundColor('#f3f4f6');
const { invoke } = window.__TAURI__.core;

let greetInputEl;
let greetMsgEl;
//let greetBtnEl;
let incantBtnEl;
let listCanonBtnEl;
let clearDiagnosticsBtnEl;
let openPreferencesBtnEl;
let ingestBtnEl;
let openLogBtnEl;
let similaritySearchBtnEl;
let panel;
let panelToggleBtn;

let prefsLoadBtn;
let prefsSaveBtn;
let prefsResetBtn;


let openaiApiKeyEl;
let prefsMainPromptTextArea;
let prefsResponseLimitTextArea;
let prefsFinalPreambleTextArea;
let prefsProseStyleTextArea;
let prefsSimilarityThreshold;
let prefsSimilarityThresholdValue;
let prefsTemperature;
let prefsTemperatureValue;
let prefsSimilarityCount;
let prefsSimilarityCountValue;

let prefsMaxHistoryItems;
let prefsMaxHistoryItemsValue;  

let prefsMaxOutputTokens;
let prefsMaxOutputTokensValue;
let prefsShuffleSimilars;
let closePreferencesBtnEl;

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
}

async function searchSimilarity() {
  const results = await invoke("search_similarity", { 
    query: editor.getText(), 
    limit: 10 
  });
  
  // Add log entries for the results
  results.forEach((result, index) => {
    addSimpleLogEntry({ 
      id: Date.now() + "_" + index,
      timestamp: Date.now(),
      message: `<div>
        <div class='border-l-[4px] border-amber-300 pl-2 pr-8 text-pretty font-["InputMono"]'>${result.chunk_text}</div>
        <div class='mt-2 px-2 py-1 rounded-sm bg-gray-800 w-fit'>${result.similarity_score}</div>
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
  let wasDisabled = false;
  
  const loadingInterval = setInterval(() => {
    dots = (dots + 1) % 8;
    greetMsgEl.textContent = `Emanating${'.'.repeat(dots)}`;
  }, 250);
  
  // disable the nudge charm while we're emanating...
  let actionItem = editor.extensionManager.extensions.find(extension => extension.name === 'inlineActionItem');
  if (actionItem) {
    if (actionItem.options.disabled === false) {
      wasDisabled = false;
      
      // Disable the extension temporarily..to avoid it appearing before emanation concludes..
      actionItem.options.disabled = true;
      addSimpleLogEntry({
        id: "",
        timestamp: Date.now(),
        message: 'InlineActionItem extension disabled during emanation? '+actionItem.options.disabled,
        level: 'debug'
      });
    } else {
      wasDisabled = true;
      addSimpleLogEntry({
        id: "",
        timestamp: Date.now(),
        message: 'InlineActionItem extension was already disabled: '+actionItem.options.disabled,
        level: 'debug'
      });
    }
  }  
  
  invoke("completion_from_context", { input: editor.getText() })
  .then(([content, timing]) => {
    clearInterval(loadingInterval);
    
    greetMsgEl.textContent = 'Emanation Complete';
    //console.log("Completion content:", content);
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
    if (wasDisabled === false) {
      setTimeout(() => {
        actionItem.options.disabled = false;
        addSimpleLogEntry({
          id: "",
          timestamp: Date.now(),
          message: 'InlineActionItem extension re-enabled after completion and is now => '+actionItem.options.disabled,
          level: 'debug'
        });
      }
      , 5000);
    }
  })
  .catch((err) => {
    clearInterval(loadingInterval);
    greetMsgEl.textContent = 'Error occurred '+err;
    console.error(err);
  });
}

window.addEventListener("DOMContentLoaded", async () => {
  //create();
    invoke("set_logger_app_data_path", {}).then((res) => {
    //console.log('Logger App Data Path:', res);
    // invoke("simple_log_message", { message: 'Logger App Data Path: '+res, id: "tracker", level: "info" }).then((res) => {
    // });
    // addSimpleLogEntry({
    //   id: "",
    //   timestamp: Date.now(),
    //   message: 'Logger App Data Path fn release: '+res,
    //   level: 'info'
    // });
    // invoke("get_logger_path", {}).then((res) => {
    //   console.log('Logger Path:', res);
    //   invoke("simple_log_message", { message: 'JS Logger Path: '+res, id: "tracker", level: "debug" }).then((res) => {
    //     //console.log('simple_log_emissions', res);
    //   });
    // });
  });

  let actionItem = editor.extensionManager.extensions.find(extension => extension.name === 'inlineActionItem');
  let nudgeButton = document.querySelector("#nudge-inline-action-item");
  if (actionItem) {
    const disabledOption = actionItem.options.disabled;
    if (disabledOption) {
      nudgeButton.classList.add("button-out"); // disabled is true (deactivated)
    } else {
      nudgeButton.classList.add("button-in"); // disabled is false (activated)
    }
  } else {
    console.error("InlineActionItem extension not found");
    addSimpleLogEntry({
      id: "",
      timestamp: Date.now(),
      message: 'InlineActionItem extension not found',
      level: 'error'
    });
  }
  nudgeButton.addEventListener("click", () => {
    // Access the InlineActionItem extension
    let actionItem = editor.extensionManager.extensions.find(extension => extension.name === 'inlineActionItem');
    if (actionItem) {
      const disabledOption = actionItem.options.disabled;
      if (disabledOption) {
        // Enable the extension
        actionItem.options.disabled = false;
        nudgeButton.classList.remove("button-out");
        nudgeButton.classList.add("button-in");
        nudgeButton.classList.add("enabled");
        addSimpleLogEntry({
          id: "",
          timestamp: Date.now(),
          message: 'InlineActionItem extension disabled? '+actionItem.options.disabled,
          level: 'info'
        });
      } else {
        // Disable the extension
        actionItem.options.disabled = true;
        nudgeButton.classList.remove("button-in");
        nudgeButton.classList.add("button-out");
        nudgeButton.classList.add("enabled");
        addSimpleLogEntry({
          id: "",
          timestamp: Date.now(),
          message: 'InlineActionItem extension disabled? '+actionItem.options.disabled,
          level: 'info'
        });
      }
    } else {
      console.error("InlineActionItem extension not found");
      addSimpleLogEntry({
        id: "",
        timestamp: Date.now(),
        message: 'InlineActionItem extension not found',
        level: 'error'
      });
    }
  });    
  
  // PREFERENCES PANEL
  prefsMainPromptTextArea = document.querySelector("#prefs-main-prompt");
  prefsMainPromptTextArea.addEventListener("dblclick", () => {
    prefsMainPromptTextArea.select();
  });
  prefsResponseLimitTextArea = document.querySelector("#prefs-response-limit");
  prefsResponseLimitTextArea.addEventListener("dblclick", () => {
    prefsResponseLimitTextArea.select();
  });
  prefsFinalPreambleTextArea = document.querySelector("#prefs-final-preamble");
  prefsFinalPreambleTextArea.addEventListener("dblclick", () => {
    prefsFinalPreambleTextArea.select();
  });
  prefsProseStyleTextArea = document.querySelector("#prefs-prose-style");
  prefsProseStyleTextArea.addEventListener("dblclick", () => {
    prefsProseStyleTextArea.select();
  });
  
  prefsTemperature = document.querySelector("#prefs-temperature");
  prefsTemperatureValue = document.querySelector("#prefs-temperature-value");
  prefsTemperature.addEventListener("input", () => {
    prefsTemperatureValue.textContent = (prefsTemperature.value);
  });
  
  prefsSimilarityThreshold = document.querySelector("#prefs-similarity-treashold");
  prefsSimilarityThresholdValue = document.querySelector("#prefs-similarity-treashold-value");
  
  prefsShuffleSimilars = document.querySelector("#prefs-shuffle-similars");
  
  prefsSimilarityCount = document.querySelector("#prefs-similarity-count");
  prefsSimilarityCountValue = document.querySelector("#prefs-similarity-count-value");
  prefsSimilarityCount.addEventListener("input", () => {
    prefsSimilarityCountValue.textContent = prefsSimilarityCount.value;
  });
  
  prefsMaxOutputTokens = document.querySelector("#prefs-max-output-tokens");
  prefsMaxOutputTokensValue = document.querySelector("#prefs-max-output-tokens-value");
  prefsMaxOutputTokens.addEventListener("input", () => {
    prefsMaxOutputTokensValue.textContent = prefsMaxOutputTokens.value;
  });
  
  prefsMaxHistoryItems = document.querySelector("#prefs-max-history-items");
  prefsMaxHistoryItemsValue = document.querySelector("#prefs-max-history-items-value");
  prefsMaxHistoryItems.addEventListener("input", () => {
    prefsMaxHistoryItemsValue.textContent = prefsMaxHistoryItems.value;
  });
  
  
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  //greetBtnEl = document.querySelector("#greet-btn");
  //greetBtnEl.addEventListener("click", searchSimilarity);
  // greetBtnEl.addEventListener("click", greet);
  incantBtnEl = document.querySelector("#incant-btn");
  incantBtnEl.addEventListener("click", completionFromContext);
  listCanonBtnEl = document.querySelector("#list-canon-btn");
  listCanonBtnEl.addEventListener("click", showCanonList);
  similaritySearchBtnEl = document.querySelector("#similarity-search-btn");
  similaritySearchBtnEl.addEventListener("click", searchSimilarity);
  ingestBtnEl = document.querySelector("#ingest-btn");
  ingestBtnEl.addEventListener("click", openDialogForIngestion);
  clearDiagnosticsBtnEl = document.querySelector("#clear-diagnostics-btn");
  clearDiagnosticsBtnEl.addEventListener("click", () => {
    diagnostics.commands.clearContent();
  });
  closePreferencesBtnEl = document.querySelector("#prefs-close-btn");
  closePreferencesBtnEl.addEventListener("click", () => {
    panel.classList.remove('open');
  });
  panel = document.getElementById('side-panel');
  
  openaiApiKeyEl = document.querySelector("#openai-api-key-btn");
  openaiApiKeyEl.addEventListener("click", () => {
    invoke("load_openai_api_key_from_keyring", {}).then((res) => {
      openaiApiKeyEl.value = res;
      addSimpleLogEntry({
        id: "",
        timestamp: Date.now(),
        message: 'OpenAI API Key: '+res,
        level: 'debug'
      });
    });
  });

  prefsLoadBtn = document.querySelector("#prefs-load-btn");
  prefsLoadBtn.addEventListener("click", () => {
    invoke("load_preferences").then((res) => {
      console.log('Preferences Loaded:', res);
      const resJson = JSON.stringify(res, null, 2);
      addSimpleLogEntry({
        id: "",
        timestamp: Date.now(),
        message: 'Preferences loaded<br/>'+resJson,
        level: 'info'
      });
      prefsMainPromptTextArea.value = res.main_prompt;
      prefsResponseLimitTextArea.value = res.response_limit;
      prefsFinalPreambleTextArea.value = res.final_preamble;
      prefsProseStyleTextArea.value = res.prose_style;
    });
    
    invoke("prefs_file_path").then((res) => { 
      const resJson = JSON.stringify(res, null, 2);
      console.log("prefs file path", res);
      addSimpleLogEntry({
        id: "",
        timestamp: Date.now(),
        message: resJson,
        level: "info"
      });
    });
    
  });
  
  // Update the ghost text span when the range input value changes
  prefsSimilarityThreshold.addEventListener("input", () => {
    prefsSimilarityThresholdValue.textContent = (prefsSimilarityThreshold.value / 100).toFixed(2);
  });
  
  
  prefsSaveBtn = document.querySelector("#prefs-save-btn");
  prefsSaveBtn.addEventListener("click", () => {
    //console.log("what's this ->", prefsSimilarityThreshold.value);
    // Convert the string "true"/"false" to an actual boolean
    const shuffleSimilarsValue = prefsShuffleSimilars.checked;
    const openaiApiKey = document.querySelector("#openai-api-key").value;
    addSimpleLogEntry({
      id: "",
      timestamp: Date.now(),
      message: 'openaiApiKey is '+openaiApiKey,
    });
    invoke("save_openai_api_key_to_keyring", { key:openaiApiKey } ).then((res) => {
      console.log('OpenAI API Key Saved:', res);
    }).catch((error) => {
      console.error('Failed to save OpenAI API Key:', error); 
      addSimpleLogEntry({
        id: "",
        timestamp: Date.now(),
        message: 'Failed to save OpenAI API Key: '+error,
        level: 'error'
      });
    });
    // addSimpleLogEntry({
    //   id: "",
    //   timestamp: Date.now(),
    //   message: 'shuffleSimilarsValue is '+shuffleSimilarsValue,
    //   level: 'debug'
    // });
    invoke("update_preferences", {
      responselimit: prefsResponseLimitTextArea.value,
      mainprompt: prefsMainPromptTextArea.value,
      finalpreamble: prefsFinalPreambleTextArea.value, 
      prosestyle: prefsProseStyleTextArea.value,
      similaritythreshold: prefsSimilarityThreshold.value,
      shufflesimilars: shuffleSimilarsValue, 
      similaritycount: prefsSimilarityCount.value,
      maxhistory: prefsMaxHistoryItems.value,
      maxtokens: prefsMaxOutputTokens.value,
      temperature: prefsTemperature.value
    }).then((res) => {
      console.log('Preferences Saved:', res);
      greetMsgEl.textContent = 'Preferences saved';
      addSimpleLogEntry({
        id: "",
        timestamp: Date.now(),
        message: 'Preferences saved<br/>'+JSON.stringify(res, null, 2),
        level: 'debug'
      });
    }).catch((error) => {
      console.error('Failed to save preferences:', error);
      greetMsgEl.textContent = 'Failed to save preferences: '+error;
      alert('Failed to save preferences:', error);
    });
    console.log("Saving preferences");
  });
  
  prefsResetBtn = document.querySelector("#prefs-reset-btn");
  prefsResetBtn.addEventListener("click", () => {
    invoke("reset_preferences").then((res) => {
      console.log('Preferences Reset:', res);
      prefsMainPromptTextArea.value = res.main_prompt;
      prefsResponseLimitTextArea.value = res.response_limit;
      prefsFinalPreambleTextArea.value = res.final_preamble;
      prefsProseStyleTextArea.value = res.prose_style;
      prefsMaxHistoryItems.value = res.max_history;
      prefsMaxHistoryItemsValue.textContent = res.max_history;
      prefsMaxOutputTokens.value = res.max_output_tokens;
      prefsMaxOutputTokensValue.textContent = res.max_output_tokens;
      prefsTemperature.value = res.temperature;
      prefsTemperatureValue.textContent = res.temperature;
      prefsShuffleSimilars.checked = res.shuffle_similars;
      prefsSimilarityThreshold.value = res.similarity_threshold * 100;
      prefsSimilarityThresholdValue.textContent = res.similarity_threshold;
      prefsSimilarityCount.value = res.similarity_count;
      prefsSimilarityCountValue.textContent = res.similarity_count;
    });
  });
  
  
  panelToggleBtn = document.getElementById('panel-toggle-btn');
  
  panelToggleBtn.addEventListener('click', () => {
    console.log('Toggling panel');
    // console.log('Panel before:', panel.classList.contains('open')); 
    invoke("load_preferences").then((res) => {
      console.log('Preferences Loaded:', res);
      addSimpleLogEntry({
        id: "",
        timestamp: Date.now(),
        message: 'Preferences loaded<br/>'+JSON.stringify(res, null, 2),
        level: 'info'
      });
      prefsMainPromptTextArea.textContent = res.main_prompt;
      prefsResponseLimitTextArea.textContent = res.response_limit;
      prefsFinalPreambleTextArea.textContent = res.final_preamble;
      prefsProseStyleTextArea.textContent = res.prose_style;
      prefsShuffleSimilars.checked = res.shuffle_similars;
      prefsSimilarityThreshold.value = res.similarity_threshold * 100;
      prefsSimilarityThreshold.textContent = res.similarity_threshold;
      prefsSimilarityThresholdValue.textContent = res.similarity_threshold;
      prefsSimilarityCount.value = res.similarity_count;
      prefsSimilarityCountValue.textContent = res.similarity_count;
      prefsMaxHistoryItems.value = res.max_history;
      prefsMaxOutputTokens.value = res.max_output_tokens;
      prefsTemperature.value = res.temperature;
      prefsTemperatureValue.textContent = res.temperature;
      
      panel.classList.toggle('open');
      panelToggleBtn.classList.toggle('open');
    });
    
  });
  
  openLogBtnEl = document.querySelector("#open-log-btn");
  openLogBtnEl.addEventListener("click", () => {
    const currentWindow = getCurrentWebviewWindow()
    console.log(currentWindow);
    invoke("get_logger_path", {}).then((res) => {
      console.log('Logger Path:', res);
      const logPath = res;
      const encodedLogPath = encodeURIComponent(logPath);
      const webview = new WebviewWindow('view-log-window-label', {
        url: `view-log.html?logPath=${encodedLogPath}`, // URL to load
        title: 'Ghostwriter Log Viewer',
        width: 800,
        height: 600,
        resizable: true,
        fullscreen: false,
        decorations: true, // window decorations (title bar, borders)
        transparent: false,
        center: true
      })
      webview.once('tauri://created', function () {
        // webview successfully created
        console.log("created");
      });
      webview.once('tauri://error', function (e) {
        // an error happened creating the webview
        console.log("woops", e)
        addSimpleLogEntry({
          id: "",
          timestamp: Date.now(),
          message: 'Error opening log viewer: '+JSON.stringify(e, null, 2)+". (Window is probably already open and buried behind other windows.)",
          level: 'error'
        });
      });
    });
  });
  
  
  
  
  // Add Tauri event listener here
  let unlistenSimpleLogMessageFn;
  let unlistenRichLogMessageFn;
  let unlistenProgressIndicatorUpdateFn;
  let unlistenProgressIndicatorLoadFn;
  let unlistenOpenFileDialogForIngestFn;
  let unlistenCanonListFn;
  let unlistenFileSaveDialogFn;
  let unlistenPrefsLoadFn;
  let unlistenPrefsSaveFn;
  let unlistenPrefsResetFn;
  
  try {
    unlistenFileSaveDialogFn = await listen('file-save-dialog', (event) => {
      console.log('Received file save event:', event);
      save({
        filters: [{
          name: 'Text',
          extensions: ['txt', 'md', 'mdx']
        }]
      }).then((result) => {
        console.log('File save dialog result:', result);
        editor.getText().then((content) => {
          invoke("save_to_file", {
            file_path: result,
            content: content
          }).then((res) => {
            console.log('File saved:', res);
            addSimpleLogEntry({
              id: Date.now(),
              timestamp: Date.now(),
              message: 'File saved: '+res,
              level: 'info'
            });
          }).catch((error) => {
            addSimpleLogEntry({
              id: Date.now(),
              timestamp: Date.now(),
              message: 'Failed to save file: '+error,
              level: 'error'
            });
          });
        });
      });
    });
  } catch (error) {
    console.error('Failed to setup event listener:', error);
  }

    
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
    });  
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
        greetMsgEl.textContent = 'Ingestion Completed for '+event.payload.current_file+' with '+event.payload.total_steps+' chunks.';
        window.timeout(() => {
          greetMsgEl.textContent = 'Ingestion Completed';
        }
        , 5000);
      }
    });
  } catch (error) {
    console.error('Failed to setup event listener:', error);
  }
  
  invoke("load_preferences").then((res) => {
    console.log('Preferences Loaded:', res);
  });
  
  invoke("simple_log_message", { message: 'Ghostwriter Is Up.', id: "tracker", level: "info" }).then((res) => {
    console.log('simple_log_emissions', res);
  });
  
  
  invoke("get_canon_info", {}).then((res) => {
    console.log('Canon Info:', res);
    if (typeof res === 'object' && res !== null) {
      //console.log('Canon Info:', res);
      addSimpleLogEntry({
        id: "",
        timestamp: Date.now(),
        message: 'Canon Info: '+JSON.stringify(res, null, 2),
        level: 'info'
      });
    }
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

// Function to handle the onDelete logic
const handleRichLogEntryDelete = ({ node, getPos, editor }) => {
  const pos = getPos(); // Get the position immediately
  const doc_id = node.attrs.id; // Get the ID of the node which should be the doc_id
  
  // Show confirmation dialog
  confirm("Delete "+node.attrs.message, "Confirmation")
  .then((confirmed) => {
    if (confirmed) {
      // User confirmed, proceed with deletion
      invoke("delete_canon_entry", { docid: doc_id })
      .then((res) => {
        // Deletion successful
      })
      .catch((error) => {
        console.error("Failed to delete canon entry:", error);
        // Handle error
      });
      
      // Check if the position is valid and the node is still in the document
      if (typeof pos === 'number' && pos >= 0 && pos < editor.state.doc.content.size) {
        // The position is valid, so proceed with deleting the node
        editor.chain().focus().deleteRange({ from: pos, to: pos + 1 }).run();
      } else {
        // The position is invalid, so log an error and do nothing
        console.error('Invalid position for node:', node, pos);
      }
    } else {
      // User cancelled, do nothing
      console.log("Deletion cancelled by user");
    }
  });
};

const editor = new Editor({
  element: document.querySelector('.element'),
  autofocus: true,
  editable: true,
  extensions: [
    StarterKit,
    DynamicTextMark,
    InlineActionItem.configure({
      disabled: true,                // Disables the feature
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
          greetMsgEl.textContent = 'Error occurred '+error;
          
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
  editable: false,
  extensions: [
    StarterKit,
    //DiagnosticLogEntryNode,
    SimpleLogEntryNode,
    RichLogEntryNode.configure({
      onDelete: handleRichLogEntryDelete,
    }),
    DynamicTextMark,
    ProgressExtension,
  ],
})

async function showCanonList() {
  try {
    await invoke("list_canon_docs", { limit: 10 });
    let pluginKey = new PluginKey('inlineActionItem');
    console.log('Plugin Key:', pluginKey);
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

function addSimpleLogEntry(entry) {
  let pos = diagnostics.state.selection.from + 2
  diagnostics.commands.setTextSelection(pos)
  diagnostics.commands.insertContent({
    type: 'simpleLogEntry',
    attrs: {
      id: entry.id,
      timestamp: formatTimestamp(entry.timestamp),
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

function formatTimestamp(timestamp) {
  const date = new Date(timestamp); // Ensure timestamp is a Date object
  
  const formatted = `${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}-${String(date.getFullYear()).slice(-2)} ` +
  `${String(date.getHours()).padStart(2, '0')}:${String(date.getMinutes()).padStart(2, '0')}:${String(date.getSeconds()).padStart(2, '0')}:${String(date.getMilliseconds()).padStart(3, '0')}`;
  
  return formatted;
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
  const diagnosticsEditor = document.querySelector('#diagnostics-editor');
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
    const maxHeight = diagnosticsEditor.offsetHeight; // Get current height of diagnostics-editor
    // Check if bottom area should collapse (dragging down)
    if (newBottomHeight < MIN_HEIGHT_BOTTOM) {
      bottomArea.classList.add('collapsed');
      topArea.style.flex = '1';
      bottomArea.style.flex = '0';
      return;
    }
    
    // Check if bottom area would exceed max height (dragging up)
    // if (newBottomHeight > maxHeight) {
    //   return;
    // }
    
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

// Add this to your existing JavaScript
