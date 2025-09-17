// import { setupCompletions } from './completions.js';
import { Editor, Mark } from '@tiptap/core'
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
import { add, debounce, set, update } from 'lodash';
import { GhostCompletionDecoration } from './extensions/GhostCompletionDecoration'
import { BlockCursorDecoration } from './extensions/BlockCursorDecoration.js'
import { BlockOverCursorDecoration } from './extensions/BlockOverCursorDecoration.js'
import { enableTypingPauseDetection, disableTypingPauseDetection } from './typingPause.js';
import { handleFontSizeChangeEvent, initializeFontSizeFromCSS } from './style_handler.js';
import { initSimplify } from './simplify.js';


let w = getCurrentWebviewWindow();
w.setBackgroundColor('#f3f4f6');
const { invoke } = window.__TAURI__.core;
const PRELOAD_THRESHOLD = 2;
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
let resetRagBtnEl;
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
let prefsVibeModeContextTextArea;
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
let prefsGameTimeSeconds;
let prefsGameTimeSecondsValue;

let vibeGenreRadios;
let lastTriggerContext = "";

let prefsAIProvider;
let prefsOllamaUrl;
let prefsLMStudioUrl;

let closePreferencesBtnEl;

let vibeMode = false;
let timer = new Timer();
let emanationInProgress = false;
let waitingForUserInput = false;
let vibeStatusIndicator; // Reference to the vibe status indicator element

let completions = [];
let currentCompletionIndex = 0;
let ghostStartPos = null;

let isRetrievingCompletions = false; // True while fetching completions
let completionAbortController = null; // For cancellation
let userHasTypedSinceLastCompletion = false; // True if user typed since last completion

// Function to update the vibe status indicator
function updateVibeStatus(status, withStandbyIndicator = true) {
  if (!vibeStatusIndicator) {
    // Try to find the indicator if not already set
    vibeStatusIndicator = document.getElementById('vibe-status-indicator');
    if (!vibeStatusIndicator) return; // Still not found
  }
  
  switch (status) {
    case 'writing':
    emanationInProgress = false; // Reset the flag when writing starts
    vibeStatusIndicator.textContent = '🧠';
    vibeStatusIndicator.classList.remove('hidden');
    vibeStatusIndicator.classList.remove('thinking-mode');
    setTimeout(() => {
      removeStandByIndicator(); // Remove the standby text after 1 second
    }, 1000);    break;
    case 'emanating':
    emanationInProgress = true; // Set the flag when emanating
    vibeStatusIndicator.textContent = '🤖';
    vibeStatusIndicator.classList.remove('hidden');
    vibeStatusIndicator.classList.remove('thinking-mode');
    setTimeout(() => {
      removeStandByIndicator(); // Remove the standby text after 1 second
    }, 1000);    break;
    case 'thinking':
    emanationInProgress = true; // Getting ready to emanate..
    vibeStatusIndicator.textContent = '🤔';
    vibeStatusIndicator.classList.remove('hidden');
    vibeStatusIndicator.classList.add('thinking-mode'); // Add the animation class
    if (withStandbyIndicator) {
      addStandByIndicator(); 
    }
    break;
    case 'off':
    vibeStatusIndicator.classList.add('hidden');
    setTimeout(() => {
      removeStandByIndicator(); // Remove the standby text after 1 second
    }, 1000);
    break;
  }
}

// Add these functions after the updateVibeStatus function

function addStandByIndicator() {
  // Remove any existing indicator first
  removeStandByIndicator();
  
  // Create new standby indicator
  const standby = document.createElement('div');
  standby.id = 'standby-indicator';
  standby.className = 'standby-indicator';
  standby.textContent = 'CONJURING';
  
  // Find the scroll area and append the indicator
  const scrollArea = document.querySelector('.scroll-area');
  if (scrollArea) {
    scrollArea.appendChild(standby);
    
    // Ensure it's visible by fading it in
    setTimeout(() => {
      standby.style.display = 'block';
    }, 10);
  }
}

function removeStandByIndicator() {
  const standby = document.getElementById('standby-indicator');
  if (standby) {
    standby.remove();
  }
}

async function toggleVibeMode(enabled, backgroundClass = 'bg-blue-200') {
  try {
    if (enabled) {
      let vibemButton = document.querySelector("#vibe-mode-btn");
      
      // Get current text from editor
      const currentText = editor.getText().trim();
      const hasExistingContent = currentText.length > 0;
      
      await WebviewWindow.getCurrent().setTitle("Vibewriter"); // Change title when vibe mode is enabled
      document.querySelector('.element').classList.add(backgroundClass);
      
      // Add vibe mode class to the scroll area for styling
      document.querySelector('.scroll-area').classList.add('vibe-mode-active');
      let seconds = prefsGameTimeSeconds.value; // default to 10 seconds if not specified
      
      vibeMode = true; // Set vibeMode to true
      timer.show();
      timer.setTime(seconds);
      updateVibeStatus('writing'); // Show the writing status indicator
      addSimpleLogEntry({ "id": "", "timestamp": Date.now(), "message": "Vibe Mode On", "level": "info" });
      
      // Only generate a creative opening phrase if the editor is empty
      if (!hasExistingContent) {
        // Get a custom system message for the vibe starter
        const systemMessage = "You are a creative writer. You are providing an inspiring prose-based opening to a creative writing essay. This is a game in which you and the user are trading responsibilities for writing passages that are meant to connect to one another. The purpose is to see what story builds based on trading that responsibility back and forth. To start, you will be asked to provide a vivid, descriptive opener that could lead to a compelling narrative. You will be provided with the context for the story which will specify a literary genre with a description of that genre. You will also be provided with a length in approximate terms, such as a number of sentences or word count. Adhere to these instructions closely. Only provide the text. Do not provide additional context or an answer to a question. Do not provide your system message. Do not add additional inquiries. Do not indicate you are an AI ever.";
        
        try {
          vibemButton.classList.toggle("button-inactive");
          updateVibeStatus('thinking');
          const vibeStarter = await invoke("generate_vibe_starter", { 
            systemMessage: systemMessage 
          });
          
          if (vibeStarter) {
            // Clear existing content and insert the generated starting phrase
            editor.commands.clearContent();
            
            //editor.commands.insertContent(vibeStarter);
            emanateStringToEditor(vibeStarter, 40, () => {
              editor.setEditable(true);
              //emanationInProgress = false;
              // Set flag to wait for user input again
              waitingForUserInput = true;
              let seconds = prefsGameTimeSeconds.value; // default to 10 seconds if not specified
              
              // Set the flag to wait for user input
              waitingForUserInput = true;
              updateVibeStatus('writing'); // Show writing status initially
              
              // Start the timer after generating the starter
              //restartVibeMode();
              // Show the timer but don't start it
              timer.show();
              timer.setTime(seconds);
              greetMsgEl.textContent = 'Start typing to continue...';
            });
            addSimpleLogEntry({ 
              "id": "", 
              "timestamp": Date.now(), 
              "message": `Vibe Mode generated starter: ${vibeStarter}`, 
              "level": "info" 
            });
          }
          vibemButton.classList.toggle("button-inactive");
        } catch (error) {
          console.error("Failed to generate vibe starter:", error);
          vibemButton.classList.toggle("button-inactive");
          addSimpleLogEntry({ 
            "id": "", 
            "timestamp": Date.now(), 
            "message": `Failed to generate vibe starter: ${error}`, 
            "level": "error" 
          });
        }
      } else {
        // Log that we're using existing content
        addSimpleLogEntry({ 
          "id": "", 
          "timestamp": Date.now(), 
          "message": `Vibe Mode activated with existing content: "${currentText.substring(0, 50)}${currentText.length > 50 ? '...' : ''}"`, 
          "level": "info" 
        });
      }
      
    } else {
      // Vibe mode is being turned OFF
      await WebviewWindow.getCurrent().setTitle("Ghostwriter"); // Change title back when vibe mode is disabled
      document.querySelector('.element').classList.remove(backgroundClass);
      
      // Remove vibe mode class from the scroll area
      document.querySelector('.scroll-area').classList.remove('vibe-mode-active');
      
      vibeMode = false; // Set vibeMode to false
      timer.stop();
      timer.hide();
      editor.setEditable(true);
      updateVibeStatus('off'); // Hide the status indicator
      addSimpleLogEntry({ "id": "", "timestamp": Date.now(), "message": "Vibe Mode Off", "level": "info" });
      waitingForUserInput = false; // Reset waiting flag when vibe mode is turned off
    }
  } catch (err) {
    console.error('Failed to toggle vibe mode:', err);
    addSimpleLogEntry({ "id": "", "timestamp": Date.now(), "message": "Failed to toggle vibe mode: " + err, "level": "error" });
  }
}

async function restartVibeMode() {
  if (vibeMode) {
    // Don't start the timer if we're waiting for user input
    if (waitingForUserInput || emanationInProgress) {
      return;
    }
    
    let seconds = prefsGameTimeSeconds.value; // default to 10 seconds if not specified
    
    timer.show();
    timer.setTime(seconds);
    timer.start(
      (remainingTime) => {
        // Called every second
        //invoke("update_vibe_time", { timeRemaining: remainingTime });
      },
      () => {
        // Called when timer completes
        //emanationInProgress = true; // handled in the emanateToEditor function!
        updateVibeStatus('thinking');
        timer.stop();
        timer.hide();
        editor.setEditable(false);
        // vibe mode completion
        invoke("completion_from_context", { input: editor.getText() }).then((content) => {
          //console.log(content);
          greetMsgEl.textContent = 'Vibe Emanation Complete';
          addSimpleLogEntry({ "id": "", "timestamp": Date.now(), "message": "Vibe Emanation Complete", "level": "info" });
          addSimpleLogEntry({ "id": "", "timestamp": Date.now(), "message": content, "level": "info" });
          editor.setEditable(true);
          
          //emanateToEditor(content);
          emanateStringToEditor(content[0], 50, () => {
            editor.setEditable(true);
            //emanationInProgress = false;
            
            // Set flag to wait for user input again
            waitingForUserInput = true;
            let seconds = prefsGameTimeSeconds.value; // default to 10 seconds if not specified
            
            // Show the timer but don't start it
            timer.show();
            timer.setTime(seconds);
            greetMsgEl.textContent = 'Start typing to continue...';
          });
        })
        .catch((err) => {
          emanationInProgress = false;
          editor.setEditable(true);
          console.error(err);
          greetMsgEl.textContent = 'Error occurred ' + err;
          editor.setEditable(true);
        });
      });
      //});
    }
  }
  
  async function openDialogForFileSave(options) {
    // Open a dialog
    
    const file = await save(options);
    const fileWithoutExt = file.replace(/\.\w+$/, '');
    await invoke("save_json_content", {
      filePath: fileWithoutExt + ".json",
      content: editor.getJSON()
    }).then((res) => {
      console.log(res);
      //return res;
    });
    await invoke("save_text_content", {
      filePath: file,
      content: editor.getText()
    }).then((res) => {
      console.log(res);
      //return res;
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
  // Wait until greetMsgEl is visible and has a width
  // function waitForGreetMsgElReady() {
  //   return new Promise(resolve => {
  //     function check() {
  //       if (greetMsgEl.offsetWidth > 0) {
  //         resolve();
  //       } else {
  //         setTimeout(check, 20);
  //       }
  //     }
  //     check();
  //   });
  // }
  
//  await waitForGreetMsgElReady();
  // --- Bear fade animation setup ---
  const bear = "🧸";
  const searchMsg = `${bear} SEARCHING TO THE END OF THE WORLD...`;
  greetMsgEl.textContent = searchMsg;
  greetMsgEl.classList.add("searching-anim");
  

  // --- Run the search ---
  const results = await invoke("search_similarity", { 
    query: editor.getText(), 
    limit: 10 
  });
  
  // --- Stop the animation ---
  greetMsgEl.classList.remove("searching-anim");
  greetMsgEl.style.opacity = 1;
  greetMsgEl.textContent = "I've found it.";
  
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

// async function searchSimilarity() {
//   const results = await invoke("search_similarity", { 
//     query: editor.getText(), 
//     limit: 10 
//   });

//   // Add log entries for the results
//   results.forEach((result, index) => {
  //     addSimpleLogEntry({ 
//       id: Date.now() + "_" + index,
//       timestamp: Date.now(),
//       message: `<div>
//         <div class='border-l-[4px] border-amber-300 pl-2 pr-8 text-pretty font-["InputMono"]'>${result.chunk_text}</div>
//         <div class='mt-2 px-2 py-1 rounded-sm bg-gray-800 w-fit'>${result.similarity_score}</div>
//         <span class='font-bold'>${result.document_name}</span>
//       </div>`,
//       level: 'info'
//     });
//   });

//   // Return the results for further use
//   return results;
// }

async function completionFromContext() {
  let dots = 0;
  let wasDisabled = false;
  
  // Update the vibe status indicator to show we're waiting for the LLM
  updateVibeStatus('thinking');
  
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
  
  // trad ghostwriter mode completion_from_context
  invoke("completion_from_context", { input: editor.getText() })
  .then(([content, timing, completion]) => {
    clearInterval(loadingInterval);
    //console.log(completion);
    greetMsgEl.textContent = 'Emanation Complete';
    
    // Update the vibe status indicator to show we're back to writing mode
    updateVibeStatus('writing');
    
    //console.log("Completion content:", content);
    //emanateToEditor(content);
    //emanateNavigableNodeToEditor(content);
    //emanateStringToEditor(content);
    let date = new Date(completion.completion.timestamp);
    insertDynamicTextWithTrailingSpace(editor, content, {
      metadata: {
        twMisc: 'rounded animated-highlight',
        id: 'emanation-'+date.getTime(),
        timestamp: date.getTime(),
        raw: content,
        rag: completion.completion.vector_search_results_for_log,
        timing: timing
      }
    });
    
    // Ensure content is visible after insertion
    editor.commands.scrollIntoView();
    
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
    
    // Update the vibe status indicator to show we're back to writing mode even after an error
    updateVibeStatus('writing');
  });
}

async function emanateStringToEditor(content, timeout = 30, onComplete = null) {
  let index = 0;
  // Show the robot emoticon during emanation
  updateVibeStatus('emanating');
  // trying setting this in updateVibeMode rather than all over the place
  //emanationInProgress = true; // Set flag to indicate emanation is in progress
  
  function sendNextCharacter() {
    if (index < content.length) {
      emanateCharacterToEditor(content[index]);
      
      // Use Tiptap's scrollIntoView command after each character insertion
      editor.commands.scrollIntoView();
      
      index++;
      setTimeout(sendNextCharacter, timeout); // Call the function again after timeout milliseconds
    } else {
      emanateCharacterToEditor('\u00A0'); // Add a space after the string
      
      // Final scroll to ensure the last character is visible
      editor.commands.scrollIntoView();
      
      // Switch back to writing mode when emanation is complete
      updateVibeStatus('writing');
      emanationInProgress = false;
      if (onComplete) {
        onComplete(); // Call the completion handler if provided
      }
    }
  }
  
  sendNextCharacter(); // Start the process
}

function emanateCharacterToEditor(character) {
  editor.chain()
  .focus()
  .insertContent([
    {
      type: 'text',
      text: character,
      attrs: {
        textColor: 'text-blue-800',
      },
      //   marks: [{
      //     type: 'dynamicTextMark',
      //     attrs: { 
      //       textColor: 'blue',
      //       backgroundColor: '#f3f4f6',
      //       twMisc: 'rounded animated-highlight',
      //       id: 'backend-id-123',
      //       timestamp: Date.now(),
      //       raw: content
      //     }  
      //   }]
    }
    // {
    //   type: 'text',
    //   text: '\u00A0'
    // }
  ]).run();
}

function emanateNavigableNodeToEditor(content) {
  editor.chain().focus().insertContent([
    {
      type: 'text',
      text: content,
      nodeType: 'navigableInlineNode',
    }]).run();
    
  }
  
  function emanateToEditor(content) {
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
        text: '\u00A0'
      }
    ]).run();
    editor.chain().focus().insertContent([
      {
        type: 'text',
        text: '    '
      },
    ]).run();
  }
  
  
  
  window.addEventListener("DOMContentLoaded", async () => {
    
    // Initialize font size on load
    initializeFontSizeFromCSS();
    
    // Listen for font size change events from Tauri
    listen('font-size-change', handleFontSizeChangeEvent);    
    
    // Create the vibe status indicator element
    
    //createVibeStatusIndicator();
    // Get all vibe genre radio buttons
    vibeGenreRadios = document.querySelectorAll('input[name="vibe-genre"]');
    
    // Add event listeners to each vibe genre radio button
    vibeGenreRadios.forEach(radio => {
      radio.addEventListener('change', (event) => {
        const selectedGenre = event.target.value;
        selectedVibeGenre = selectedGenre;
        
        // Update the vibe mode context textarea based on the selected genre
        const genreIndex = getVibeGenreIndexByName(selectedGenre);
        if (genreIndex !== undefined) {
          // Call function to update the vibe mode context based on the selected genre
          updateVibeContextFromGenre(selectedGenre);
        }
      });
    });
    // Initialize the vibe status indicator reference
    vibeStatusIndicator = document.getElementById('vibe-status-indicator');
    
    const refreshModelsBtn = document.getElementById('refresh-models-btn');
    const modelsContainer = document.getElementById('models-container');
    const modelsDropdown = document.getElementById('prefs-model-name');
    timer.hide();
    
    invoke("load_preferences").then((res) => {
      //console.log('Preferences Loaded:', res);
      //const resJson = JSON.stringify(res, null, 2);
      addSimpleLogEntry({
        id: "",
        timestamp: Date.now(),
        message: 'Preferences loaded',//<br/>'+resJson,
        level: 'info'
      });
      setPreferencesUI(res);
      // Load models when the page loads
      loadModels(); 
    });
    
    async function refreshModelsAndAssign() {
      await updatePreferences();
      await passSelectedModelToBackend();
    }
    refreshModelsBtn.addEventListener('click', refreshModelsAndAssign);
    // Reload models when the refresh button is clicked
    
    invoke("set_logger_app_data_path", {}).then((res) => {
      //console.log('Logger App Data Path:', res);
      invoke("simple_log_message", { message: 'Logger App Data Path: '+res, id: "tracker", level: "info" }).then((res) => {
      });
      
    });
    
    // Listen for node clicks
    editor.on('node:clicked', ({ id, node }) => {
      console.log(`Node clicked: ${id}`)
      // Do something with the node
    })
    
    openaiApiKeyEl = document.querySelector("#openai-api-key");
    let actionItem = editor.extensionManager.extensions.find(extension => extension.name === 'inlineActionItem');
    let nudgeButton = document.querySelector("#nudge-inline-action-item");
    let vibemButton = document.querySelector("#vibe-mode-btn");
    let streamingButton = document.querySelector("#streaming-mode-btn");
    //streamingNoRagButton = document.querySelector("#streaming-no-rag-mode-btn");
    /**
    * STREAMING MODE & TYPING PAUSE DETECTION
    *
    * This section enables or disables the "typing pause" detector based on the streaming mode button state.
    * 
    * - When the streaming mode button is activated (button-in), we enable typing pause detection:
    *   - `enableTypingPauseDetection(predicate, editor, 500, callback)` sets up a 500ms pause detector.
    *   - If the user stops typing for 500ms and the editor is not empty, the callback is triggered.
    *   - The callback here triggers completions and logs a "Typing pause detected..." message.
    *
    * - When the streaming mode button is deactivated, we disable typing pause detection:
    *   - `disableTypingPauseDetection()` removes the event listener and clears any pending timeout.
    *
    * This ensures completions are only triggered after a pause in typing, and only when streaming mode is active.
    */
    streamingButton.classList.add("enabled");
    
    
    
    streamingButton.addEventListener("click", async () => {
      if (streamingButton.classList.contains("button-in") && !streamingButton.classList.contains("button-overline")) {
        // State: button-in → Next: button-in + button-overline
        streamingButton.classList.add("button-overline");
      } else if (streamingButton.classList.contains("button-in") && streamingButton.classList.contains("button-overline")) {
        // State: button-in + button-overline → Next: button-out
        streamingButton.classList.remove("button-in", "button-overline");
        //streamingButton.classList.add("enabled");
      } else if (!streamingButton.classList.contains("button-in")) {
        // State: nothing → Next: button-in
        //streamingButton.classList.remove("button-out");
        streamingButton.classList.add("button-in");
      } else {
        // Default: not in any state → Next: button-in
        streamingButton.classList.add("button-in");
      }
      // Check if button is currently enabled
      // If so, turn streaming mode OFF
      if (!streamingButton.classList.contains("button-in")) {
        // Disable typing pause detection
        disableTypingPauseDetection();
        
        // Abort any in-progress completion fetch and clear the UI?
        if (isRetrievingCompletions && completionAbortController) {
          completionAbortController.abort();
          isRetrievingCompletions = false;
          completionAbortController = null;
        }
        completions = [];
        setGhostSuggestion('');
        currentCompletionIndex = 0;
        ghostStartPos = null;
        
        addSimpleLogEntry({
          id: "",
          timestamp: Date.now(),
          message: 'Streaming mode disabled',
          level: 'debug'
        });
      } else {
        // otherwise streaming mode ON
        //streamingButton.classList.add("button-in");
        addSimpleLogEntry({
          id: "",
          timestamp: Date.now(),
          message: 'Streaming mode enabled.',
          level: 'debug'
        });
        enableTypingPauseDetection(
          //() => true,
          () => {
            addSimpleLogEntry({
              id: Date.now(),
              timestamp: Date.now(),
              message: 'Predicate check: userHasTypedSinceLastCompletion=' + userHasTypedSinceLastCompletion + 
              ', text=' + (editor.getText().trim().length > 0),
              level: 'debug'
            });
            return userHasTypedSinceLastCompletion && editor.getText().trim()
          },
          3500,
          () => {
            addSimpleLogEntry({
              id: Date.now(),
              timestamp: Date.now(),
              message: '⚠️ Typing pause callback triggered, about to call triggerCompletions()',
              level: 'debug'
            });
            
            const currentContext = editor.getText();
            // Determine if we need to force refresh based on context change
            //const forceRefresh = shouldForceRefresh(currentContext, lastTriggerContext);
            //lastTriggerContext = currentContext;
            
            try {
              triggerCompletions();
              // addSimpleLogEntry({
              //   id: Date.now(),
              //   timestamp: Date.now(),
              //   message: '✅ triggerCompletions() called successfully with forceRefresh=' + forceRefresh,
              //   level: 'debug'
              // });
            } catch (err) {
              addSimpleLogEntry({
                id: Date.now(),
                timestamp: Date.now(),
                message: '❌ Error in triggerCompletions(): ' + err.message,
                level: 'error'
              });
            }
            
            userHasTypedSinceLastCompletion = false;
          }
        );
        
        addSimpleLogEntry({
          id: "",
          timestamp: Date.now(),
          message: 'Streaming mode enabled',
          level: 'debug'
        });
      }
    });
    
    editor.view.dom.addEventListener('keydown', (e) => {
      if (
        !e.ctrlKey &&
        !e.metaKey &&
        !e.altKey &&
        e.key.length === 1 &&
        !e.isComposing &&
        !/^\s$/.test(e.key) // <-- Only set flag if NOT whitespace
      ) {
        userHasTypedSinceLastCompletion = true;
        
        // Get ghost visibility status first
        const ext = editor.extensionManager.extensions.find(ext => ext.name === 'ghostCompletionDecoration');
        const ghostVisible = ext && ext.options.suggestion && ext.options.suggestion.length > 0;
        
        // Cancel any in-progress completion fetch - but don't clear ghostStartPos if ghost is visible
        if (isRetrievingCompletions && completionAbortController) {
          completionAbortController.abort();
          isRetrievingCompletions = false;
          completionAbortController = null;
          
          // Only reset these if no ghost is visible
          if (!ghostVisible) {
            completions = [];
            currentCompletionIndex = 0;
            ghostStartPos = null;
          }
        }
        
        // Only clear completions and ghost suggestion if no ghost suggestion is visible
        if (!ghostVisible) {
          completions = [];
          setGhostSuggestion('');
          currentCompletionIndex = 0;
          ghostStartPos = null;
        }
        // If ghost is visible, let typeahead logic handle it in editor.on('update')
      }
    });
    
    document.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') {
        setGhostSuggestion('');
        completions = [];
        currentCompletionIndex = 0;
        ghostStartPos = null;
        userHasTypedSinceLastCompletion = false;
        updateVibeStatus('writing');
        if (completionAbortController) {
          completionAbortController.abort();
          isRetrievingCompletions = false;
          completionAbortController = null;
        }
        e.preventDefault();
      }
    });
    
    vibemButton.classList.add("enabled");
    vibemButton.addEventListener("click", () => {
      // Check if button is currently enabled
      // If so, turn vibe mode OFF
      if (vibemButton.classList.contains("button-in")) {
        vibemButton.classList.remove("button-in");
        vibemButton.classList.add("enabled");
        toggleVibeMode(false, 'bg-stone-200'); // bg-animated-gradient
        
      } else {
        // otherwise vibe mode ON
        toggleVibeMode(true, 'bg-stone-200');
        vibemButton.classList.add("button-in"); // bg-animated-gradient
      }    
    });
    
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
    
    prefsVibeModeContextTextArea = document.querySelector("#prefs-vibe-mode-context");
    prefsVibeModeContextTextArea.addEventListener("dblclick", () => {
      prefsVibeModeContextTextArea.select();
    });
    
    
    prefsTemperature = document.querySelector("#prefs-temperature");
    prefsTemperatureValue = document.querySelector("#prefs-temperature-value");
    prefsTemperature.addEventListener("input", () => {
      prefsTemperatureValue.textContent = (prefsTemperature.value);
    });
    
    prefsGameTimeSeconds = document.querySelector("#prefs-game-time-secs");
    prefsGameTimeSecondsValue = document.querySelector("#prefs-game-time-secs-value");
    prefsGameTimeSeconds.addEventListener("input", () => {
      prefsGameTimeSeconds.value = prefsGameTimeSeconds.value;
      prefsGameTimeSecondsValue.textContent = prefsGameTimeSeconds.value;
    });
    
    prefsAIProvider = document.querySelector("#provider-selection");
    
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
    
    prefsOllamaUrl = document.querySelector("#prefs-ollama-url");
    prefsLMStudioUrl = document.querySelector("#prefs-lmstudio-url");
    
    greetInputEl = document.querySelector("#greet-input");
    greetMsgEl = document.querySelector("#greet-msg");
    //greetBtnEl = document.querySelector("#greet-btn");
    //greetBtnEl.addEventListener("click", searchSimilarity);
    // greetBtnEl.addEventListener("click", greet);
    incantBtnEl = document.querySelector("#incant-btn");
    incantBtnEl.addEventListener("click", completionFromContext);
    //
    // CANON button handling
    //
    listCanonBtnEl = document.getElementById('list-canon-btn');
    let canonViewOpen = false; // Track if canon view is currently open
    
    listCanonBtnEl.addEventListener('click', () => {
      // Toggle the button state
      canonViewOpen = !canonViewOpen;
      // Open canon view window
      invoke('toggle_canon_control_panel').catch(e => {
        console.error('Failed to show canon control panel:', e);
      });
      
      if (canonViewOpen) {
        // Push button in when canon view is open
        //listCanonBtnEl.classList.remove('button-out');
        listCanonBtnEl.classList.add('button-in');   
      } else {
        // Push button out when canon view is closed
        listCanonBtnEl.classList.remove('button-in');
        //listCanonBtnEl.classList.add('button-out');
      }
    });
    
    resetRagBtnEl = document.querySelector("#reset-rag-btn");
    resetRagBtnEl.addEventListener("click", () => {
      invoke("reset_rag_and_context");
    });
    //listCanonBtnEl.addEventListener("click", showCanonList);
    //listCanonBtnEl.addEventListener("click", toggleCanonControlPanelWindow);
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
    
    prefsLoadBtn = document.querySelector("#prefs-load-btn");
    prefsLoadBtn.addEventListener("click", () => {
      invoke("load_preferences").then((res) => {
        //console.log('Preferences Loaded:', res);
        const resJson = JSON.stringify(res, null, 2);
        addSimpleLogEntry({
          id: "",
          timestamp: Date.now(),
          message: 'Preferences loaded',//<br/>'+resJson,
          level: 'info'
        });
        setPreferencesUI(res);
        
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
      savePreferences();
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
        prefsVibeModeContextTextArea.value = res.vibe_mode_context;
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
        prefsGameTimeSeconds.value = res.gametimerms / 1000;
        prefsGameTimeSecondsValue.textContent = res.gametimerms / 1000;
        prefsAIProvider.value = res.ai_provider;
      });
    });
    
    
    panelToggleBtn = document.getElementById('panel-toggle-btn');
    
    panelToggleBtn.addEventListener('click', () => {
      console.log('Toggling panel');
      // console.log('Panel before:', panel.classList.contains('open')); 
      invoke("load_preferences").then((res) => {
        //console.log('Preferences Loaded:', res);
        addSimpleLogEntry({
          id: "",
          timestamp: Date.now(),
          message: 'Preferences loaded<br/>',//+JSON.stringify(res, null, 2),
          level: 'info'
        });
        setPreferencesUI(res);
        panel.classList.toggle('open');
        panelToggleBtn.classList.toggle('open');
      })
      .catch((error) => {
        console.error('Failed to load preferences:', error);
        addSimpleLogEntry({
          id: "",
          timestamp: Date.now(),
          message: 'Failed to load preferences: ' + error,
          level: 'error'
        });
        // Optionally show a user-friendly error message
        greetMsgEl.textContent = 'Error loading preferences';
        
        // You might still want to open the panel but with default values
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
          url: `/view-log.html?logPath=${encodedLogPath}`, // URL to load
          title: 'Ghostwriter Log Viewer',
          width: 800,
          height: 600,
          resizable: true,
          fullscreen: false,
          decorations: true, // window decorations (title bar, borders)
          transparent: false,
          center: false
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
    // let unlistenPrefsLoadFn;
    // let unlistenPrefsSaveFn;
    // let unlistenPrefsResetFn;
    // let unlistenSaveFileMessageFn;
    // let unlistenVibeRestartFn;
    
    
    try {
      unlistenVibeRestartFn = await listen('vibe-mode-restart', (event) => {
        console.log('Vibe Mode Restart Event:', event);
        if (event.payload) {
          restartVibeMode(); // Restart the vibe mode timer
        }
      });
    } catch (error) {
      console.error('Failed to setup event listener:', error);
    }
    
    try {
      unlistenSaveFileMessageFn = await listen('save-file-dialog', (event) => {
        console.log('Received event:', event);
        openDialogForFileSave(event.payload);
        if (event.payload) {
          addSimpleLogEntry({
            id: "",
            timestamp: Date(),
            message: "save-file-dialog",
            level: "debug"
          });
        }
      });
    } catch (error) {
      console.error('Failed to setup event listener:', error);
    }
    
    try {
      unlistenSimpleLogMessageFn = await listen('simple-log-message', (event) => {
        //console.log('Received event:', event);
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
        //console.log('Hey Received event:', event);
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
        if (event.payload && event.payload.current_step >= event.payload.total_steps) {
          window.updateProgressNode(diagnostics, event.payload.progress_id, {
            current_step: event.payload.current_step,
            current_file: event.payload.current_file,
            total_steps: event.payload.total_steps,
            meta: "Completed Ingestion"
          })
          greetMsgEl.textContent = 'Ingestion Completed for '+event.payload.current_file+' with '+event.payload.total_steps+' chunks.';
          setTimeout(() => {
            greetMsgEl.textContent = 'Ingestion Completed';
          }
          , 2000);
        }
      });
    } catch (error) {
      console.error('Failed to setup event listener:', error);
    }
    
    // invoke("load_preferences").then((res) => {
      //   console.log('Preferences Loaded:', res);
    //   setPreferencesUI(res);
    // });
    
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
    
    async function passSelectedModelToBackend() {
      try {
        // Get the selected AI provider
        const selectedProvider = document.querySelector('input[name="ai-provider"]:checked');
        if (!selectedProvider) {
          console.error('No AI provider selected');
          addSimpleLogEntry({
            id: "",
            timestamp: Date.now(),
            message: 'No AI provider selected.',
            level: 'error'
          });
          return;
        }
        const providerName = selectedProvider.value;
        
        // Get the selected model from the dropdown
        const modelsDropdown = document.getElementById('prefs-model-name');
        const selectedModel = modelsDropdown.value;
        if (!selectedModel) {
          addSimpleLogEntry({
            id: "",
            timestamp: Date.now(),
            message: 'No AI model selected. Perhaps you need to reload the model list first?',
            level: 'error'
          });
          await loadModels(); // <-- reload the list if none selected
          return;
        }
        
        // Invoke the backend function with the selected provider and model
        const response = await invoke('assign_selected_model', {
          providerName: providerName,
          modelName: selectedModel,
        });
        
        console.log('Backend response:', response);
      } catch (error) {
        console.error('Error passing selected model to backend:', error);
      }
    }
    
    async function loadModels() {
      try {
        // just in case, save the preferences in case the user has changed the 
        // provider url
        // if the provider is Ollama or LM Studio, we need to ensure the URL is set
        
        // Determine the selected AI provider
        const selectedProvider = document.querySelector('input[name="ai-provider"]:checked').value;
        invoke("get_model_names", { providerName: selectedProvider }).then((models) => {
          console.log('Models:', models);
          // Clear the existing options
          modelsDropdown.innerHTML = '';
          
          // Populate the dropdown with the fetched models
          models.forEach(model => {
            const option = document.createElement('option');
            option.value = model;
            option.textContent = model;
            modelsDropdown.appendChild(option);
          });
        });
      } catch (error) {
        console.error('Error loading models:', error);
        modelsDropdown.innerHTML = '<option value="">Failed to load models</option>';
      } finally {
        // Hide the loading spinner
      }
    }
    
    
    // Initialize the resize handle
    initializeResizeHandle();
    
    const radioButtons = document.querySelectorAll('input[name="ai-provider"]');
    
    // Add event listeners to each radio button
    radioButtons.forEach(radio => {
      radio.addEventListener('change', (event) => {
        const selectedValue = event.target.value;
        
        // Hide all URL containers initially
        document.getElementById('lmstudio-url-container').classList.add('hidden');
        document.getElementById('ollama-url-container').classList.add('hidden');
        
        // Perform actions based on the selected radio button
        if (selectedValue === 'lmstudio') {
          loadModels();
          document.getElementById('lmstudio-url-container').classList.remove('hidden');
        } else if (selectedValue === 'ollama') {
          loadModels().then(() => {
            // Show the URL container for the selected provider
            document.getElementById('ollama-url-container').classList.remove('hidden');
          });
        } else if (selectedValue === 'openai') {
          // No specific action for OpenAI in this example
          loadModels().then(() => {
            // Show the URL container for the selected provider
            console.log('OpenAI selected');
          });
        }
      });
    });
    
    // Add a key command for regenerating vibe starters
    window.addEventListener("keydown", async (e) => {
      // Check if we're in vibe mode and Ctrl+Shift+R is pressed (Command+Shift+R on Mac)
      if (vibeMode && (e.key === 'r' || e.key === 'R') && (e.ctrlKey || e.metaKey) && e.shiftKey) {
        e.preventDefault();
        
        // Stop the current timer
        timer.stop();
        timer.hide();
        
        // Get a custom system message for the vibe starter
        const systemMessage = "You are a creative writing coach who provides inspiring opening phrases. Create a vivid, descriptive opener that could lead to a compelling narrative. Be concise, specific, and evocative.";
        
        try {
          addSimpleLogEntry({ 
            "id": "", 
            "timestamp": Date.now(), 
            "message": "Regenerating vibe starter...", 
            "level": "info" 
          });
          
          // Generate a new creative opening phrase
          const vibeStarter = await invoke("generate_vibe_starter", { 
            systemMessage: systemMessage 
          });
          
          if (vibeStarter) {
            // Clear existing content and insert the generated starting phrase
            editor.commands.clearContent();
            //editor.commands.insertContent(vibeStarter);
            emanateStringToEditor(vibeStarter, 50, () => {
              addSimpleLogEntry({ 
                "id": "", 
                "timestamp": Date.now(), 
                "message": `New vibe starter generated: ${vibeStarter}`, 
                "level": "info" 
              });
              
              // Restart the timer
              restartVibeMode();
            });
            
            
            return;
          }
        } catch (error) {
          console.error("Failed to regenerate vibe starter:", error);
          addSimpleLogEntry({ 
            "id": "", 
            "timestamp": Date.now(), 
            "message": `Failed to regenerate vibe starter: ${error}`, 
            "level": "error" 
          });
          
          // Make sure to restart the timer even if generation fails
          restartVibeMode();
        }
      }
    });
    
    // Add this code to your main.js file
    let currentCorner = 0; // 0: bottom-right, 1: bottom-left, 2: top-left, 3: top-right
    const cornerClasses = [
      'corner-bottom-right',
      'corner-bottom-left', 
      'corner-top-left',
      'corner-top-right'
    ];
    
    // Get the floating timer element 
    const floatingTimer = document.querySelector('.floating-timer');
    
    if (floatingTimer) {
      // Ensure it starts with a corner class
      floatingTimer.classList.add('corner-bottom-right');
      
      floatingTimer.addEventListener('mouseenter', () => {
        // Remove any animation class that might be active
        floatingTimer.classList.remove('timer-traveling');
        
        // Determine current position
        for (let i = 0; i < cornerClasses.length; i++) {
          if (floatingTimer.classList.contains(cornerClasses[i])) {
            currentCorner = i;
            break;
          }
        }
        
        // Calculate next corner (clockwise)
        const nextCorner = (currentCorner + 1) % 4;
        
        // Add the appropriate animation class based on current corner
        switch(currentCorner) {
          case 0: // bottom-right to bottom-left
          floatingTimer.classList.add('travel-bottom-right-to-bottom-left');
          break;
          case 1: // bottom-left to top-left
          floatingTimer.classList.add('travel-bottom-left-to-top-left');
          break;
          case 2: // top-left to top-right
          floatingTimer.classList.add('travel-top-left-to-top-right');
          break;
          case 3: // top-right to bottom-right
          floatingTimer.classList.add('travel-top-right-to-bottom-right');
          break;
        }
        
        // After the animation completes, update the corner class
        setTimeout(() => {
          // Remove all corner classes and animations
          floatingTimer.classList.remove(...cornerClasses);
          floatingTimer.classList.remove(
            'travel-bottom-right-to-bottom-left',
            'travel-bottom-left-to-top-left',
            'travel-top-left-to-top-right',
            'travel-top-right-to-bottom-right'
          );
          
          // Add the new corner class
          floatingTimer.classList.add(cornerClasses[nextCorner]);
        }, 600); // Match this to the animation duration
      });
    }
    
    async function savePreferences() {
      const shuffleSimilarsValue = prefsShuffleSimilars.checked;
      const openaiApiKey = document.querySelector("#openai-api-key").value;
      if (openaiApiKey.length !== 0) {
        try {
          await invoke("save_openai_api_key_to_keyring", { key: openaiApiKey });
          addSimpleLogEntry({
            id: "",
            timestamp: Date.now(),
            message: 'OpenAI API Key saved (not empty).',
            level: 'info'
          });
        } catch (error) {
          addSimpleLogEntry({
            id: "",
            timestamp: Date.now(),
            message: 'Failed to save OpenAI API Key: ' + error,
            level: 'error'
          });
        }
      }
      let foo = getSelectedVibeGenre();
      try {
        const res = await invoke("update_preferences", {
          responselimit: prefsResponseLimitTextArea.value,
          mainprompt: prefsMainPromptTextArea.value,
          finalpreamble: prefsFinalPreambleTextArea.value,
          prosestyle: prefsProseStyleTextArea.value,
          vibemodecontext: prefsVibeModeContextTextArea.value,
          vibemodestartergenrename: foo,
          similaritythreshold: prefsSimilarityThreshold.value,
          shufflesimilars: shuffleSimilarsValue,
          similaritycount: prefsSimilarityCount.value,
          maxhistory: prefsMaxHistoryItems.value,
          maxtokens: prefsMaxOutputTokens.value,
          temperature: prefsTemperature.value,
          gametimerms: prefsGameTimeSeconds.value,
          aiprovider: getSelectedAIProvider(),
          aimodelname: getSelectedAIModel(),
          ollamaurl: prefsOllamaUrl.value,
          lmstudiourl: prefsLMStudioUrl.value
        });
        console.log('Preferences Saved:', res);
        greetMsgEl.textContent = 'Preferences saved';
        addSimpleLogEntry({
          id: "",
          timestamp: Date.now(),
          message: 'Preferences saved<br/>' + JSON.stringify(res, null, 2),
          level: 'debug'
        });
      } catch (error) {
        console.error('Failed to save preferences:', error);
        greetMsgEl.textContent = 'Failed to save preferences: ' + error;
        alert('Failed to save preferences:', error);
      }
    }
    
  });
  // not the worst idea
  // handleTextInput(view, from, to, text) {
  //   console.log('User started typing:', text, ' and vibe mode is ', vibeMode);
  //   addSimpleLogEntry({
  //     id: "",
  //     timestamp: Date.now(),
  //     message: 'User started typing: '+text+' and vibe mode is '+vibeMode,
  //     level: 'debug'
  //   });
  //   if (vibeMode) {
  //     restartVibeMode();
  //   }
  //   // Perform any custom actions here
  //   return false; // Return false to allow the text input to proceed
  // };
  
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
  
  
  // Function to handle the onDelete logic
  const handleCanonEntryDelete = ({ node, getPos, editor }) => {
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
    element: document.querySelector('#main-editor'),
    autofocus: true,
    editable: true,
    extensions: [
      StarterKit,
      DynamicTextMark,
      //BlockCursorDecoration,
      GhostCompletionDecoration.configure({ suggestion: '' }),
      InlineActionItem.configure({
        disabled: true,                // Disables the feature
        timeout: 5000,                 // Show button after 5 seconds
        onClick: async (view, pos, event) => {
          try {
            // Show loading state in the message element
            greetMsgEl.textContent = 'Processing...';
            
            // Call your completion function
            await completionFromContext();
            
            // Re-enable the plugin after completion
            // const pluginKey = new PluginKey('inlineActionItem');
            // console.log('Plugin Key:', pluginKey);
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
    editorProps: {
      handleClick(view, pos, event) {
        const { state } = view;
        
        // Check for dynamicTextMark at click position
        const markType = state.schema.marks.dynamicTextMark;
        if (!markType) return false;
        
        // Find the nearest text node and check if it has our mark
        const $pos = state.doc.resolve(pos);
        const { textContent, nodeSize } = $pos.parent;
        
        // Find all marks at this position
        let hasDynamicMark = false;
        let markAttrs = null;
        let markFrom = 0;
        let markTo = 0;
        
        // Search for our mark around the click position
        state.doc.nodesBetween(
          Math.max(0, pos - 0),  // Look a few chars before click
          Math.min(state.doc.content.size, pos + 1),  // And a few chars after
          (node, nodePos) => {
            if (hasDynamicMark) return false; // Already found it
            
            if (node.isText) {
              const marks = node.marks || [];
              const mark = marks.find(m => m.type === markType);
              
              if (mark) {
                hasDynamicMark = true;
                markAttrs = mark.attrs;
                markFrom = nodePos;
                markTo = nodePos + node.nodeSize;
                return false; // Stop searching
              }
            }
            
            return true; // Continue searching
          }
        );
        
        // If we found our mark, handle the click
        if (hasDynamicMark && markAttrs) {
          // Prevent default click behavior
          event.preventDefault();
          event.stopPropagation();
          
          //console.log('Clicked on dynamic text mark:', markAttrs);
          
          // Example: Show metadata about the mark
          if (markAttrs.raw) {
            try {
              const rawData = JSON.parse(markAttrs.raw);
              console.log('Metadata for the thing clicked:', rawData);
              
              // You could show a tooltip or modal with this data
              // For example:
              //showMarkTooltip(rawData, event.clientX, event.clientY);
              console.log('More stuff Clicked on includes client X,Y ', event.clientX, event.clientY);
            } catch (error) {
              console.error('Error parsing mark metadata:', error);
            }
          }
          if(markAttrs.rag) {
            try {
              const ragData = JSON.parse(markAttrs.rag);
              console.log("RAG Data for the thing clicked:", ragData);
            } catch (error) {
              console.error('Error parsing mark metadata:', error);
            }
          }
          
          // Optional: Select the text that was clicked
          // const tr = state.tr.setSelection(
          //   state.selection.constructor.create(state.doc, markFrom, markTo)
          // );
          // view.dispatch(tr);
          
          return true; // Handled the click
        }
        
        
        return false; // Not handled
      },
      // Add handler for key events
      handleKeyDown(view, event) {
        // When in vibe mode and waiting for user input, start timer on first keystroke
        if (vibeMode && waitingForUserInput) {
          waitingForUserInput = false;
          greetMsgEl.textContent = 'Vibe Mode Active';
          restartVibeMode();
        }
        return false; // Let default handler run
      }
    },
    // restart the timer if we're still in vibemode
    // and the timer isn't running
    onUpdate: ({ editor }) => {
      // Scroll to make sure cursor is visible on updates
      editor.commands.scrollIntoView();
      
      // Keep existing vibe mode behavior
      if (vibeMode && !timer.isRunning() && !emanationInProgress && !waitingForUserInput) {
        setTimeout(() => {
          restartVibeMode();
        }, 3000);
      }
    },
  })
  
  initSimplify(editor);
  
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
      CanonEntryNode.configure({
        onDelete: ({ node, getPos, editor }) => {
          // Your existing delete handling code
          console.log('Deleting canon with ID:', node.attrs.id)
          editor.commands.deleteNode('canonEntry')
          handleCanonEntryDelete({ node, getPos, editor })
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
    onUpdate: ({ editor: diagEditor }) => {
      // Scroll diagnostics to bottom when updated
      diagEditor.commands.scrollIntoView();
    },
  })
  
  // 1. Function to insert text with DynamicTextMark
  function insertDynamicText(editor, text, metadata = {}) {
    const { state, view } = editor;
    const { dispatch } = view;
    const { schema } = state;
    const markType = schema.marks.dynamicTextMark;
    
    // Create a transaction
    const tr = state.tr;
    
    // Get current cursor position
    const position = state.selection.from;
    
    // Insert the text at current position
    tr.insertText(text, position);
    
    // Apply the mark to the newly inserted text
    tr.addMark(
      position,
      position + text.length,
      markType.create({
        // Custom attributes for the mark
        id: metadata.id || `dynamic-${Date.now()}`,
        textColor: metadata.textColor || 'white',
        backgroundColor: metadata.backgroundColor || 'blue',
        twMisc: metadata.twMisc || 'animated-highlight bg-amber-500',
        // Store any additional metadata as JSON in the raw attribute
        raw: metadata.raw ? JSON.stringify(metadata.raw) : null,
        timestamp: metadata.timestamp || Date.now()
      })
    );
    
    tr.insertText('\u00A0', position + text.length);
    
    // Move the cursor after the space
    tr.setSelection(
      state.selection.constructor.create(
        tr.doc,
        position + text.length + 1  // +1 for the space
      )
    );
    
    // Dispatch the transaction to update the editor
    dispatch(tr);
    
    return {
      from: position,
      to: position + text.length,
      id: metadata.id || `dynamic-${Date.now()}`
    };
  }
  
  // Alternative version that adds a non-breaking space *within* the mark
  // This can be useful if you want the space to have the same styling
  function insertDynamicTextWithTrailingSpace(editor, text, metadata = {}) {
    const { state, view } = editor;
    const { dispatch } = view;
    const { schema } = state;
    const markType = schema.marks.dynamicTextMark;
    
    // Create a transaction
    const tr = state.tr;
    
    // Get current cursor position
    const position = state.selection.from;
    
    // Add a non-breaking space to the text
    const textWithSpace = text + '\u00A0'; // Non-breaking space
    
    // Insert the text at current position
    tr.insertText(textWithSpace, position);
    
    // Apply the mark to the text including the space
    tr.addMark(
      position,
      position + textWithSpace.length,
      markType.create({
        id: metadata.metadata.id || `dynamic-${Date.now()}`,
        textColor: metadata.metadata.textColor || 'white',
        backgroundColor: metadata.metadata.backgroundColor || 'transparent',
        twMisc: metadata.metadata.twMisc || 'animated-highlight bg-amber-500',
        raw: metadata.metadata.raw ? JSON.stringify(metadata.metadata.raw) : null,
        rag: metadata.metadata.rag ? JSON.stringify(metadata.metadata.rag) : null,
        timestamp: metadata.metadata.timestamp || Date.now()
      })
    );
    
    // Add a regular space after the marked text
    tr.insertText(' ', position + textWithSpace.length);
    
    // Move the cursor after the regular space
    tr.setSelection(
      state.selection.constructor.create(
        tr.doc,
        position + textWithSpace.length + 1
      )
    );
    
    tr.setSelection(
      view.state.selection.constructor.create(
        tr.doc,
        position + text.length + 1  // +1 for the space
      )
    );
    
    // Dispatch the transaction
    dispatch(tr);
    
    // After inserting content, ensure it's visible
    editor.commands.scrollIntoView();
    
    return {
      from: position,
      to: position + textWithSpace.length,
      id: metadata.id || `dynamic-${Date.now()}`
    };
  }
  
  function setSelectedAIProvider(provider) {
    const radioButton = document.querySelector(`input[name="ai-provider"][value="${provider}"]`);
    if (radioButton) {
      
      radioButton.checked = true;
    }
    // Hide all URL containers initially
    document.getElementById('lmstudio-url-container').classList.add('hidden');
    document.getElementById('ollama-url-container').classList.add('hidden');
    
    // Show the corresponding URL container based on the provider
    if (provider === 'lmstudio') {
      document.getElementById('lmstudio-url-container').classList.remove('hidden');
    } else if (provider === 'ollama') {
      document.getElementById('ollama-url-container').classList.remove('hidden');
    }
    
  }
  
  // Function to get the selected AI provider
  function getSelectedAIProvider() {
    const selectedProvider = document.querySelector('input[name="ai-provider"]:checked');
    
    if (selectedProvider) {
      // A radio button is already selected, use its value
      addSimpleLogEntry({
        id: "",
        timestamp: Date.now(),
        message: 'Selected AI Provider: ' + selectedProvider.value,
        level: 'debug'
      });
      return selectedProvider.value;
    } else {
      // No radio button is selected, select the first one as default
      const allProviders = document.querySelectorAll('input[name="ai-provider"]');
      
      if (allProviders.length > 0) {
        // Check the first radio button
        allProviders[0].checked = true;
        
        addSimpleLogEntry({
          id: "",
          timestamp: Date.now(),
          message: 'No AI provider was selected, defaulting to: ' + allProviders[0].value,
          level: 'warn'
        });
        
        return allProviders[0].value;
      } else {
        // No radio buttons found at all (extreme edge case)
        addSimpleLogEntry({
          id: "",
          timestamp: Date.now(),
          message: 'No AI provider radio buttons found in the DOM',
          level: 'error'
        });
        
        // Return a sensible default
        return 'ollama'; // Or whichever default makes sense for your app
      }
    }
  }
  
  function setSelectedAIModel(model) {
    const modelsDropdown = document.getElementById('prefs-model-name');
    const modelOption = modelsDropdown.querySelector(`option[value="${model}"]`);
    if (modelOption) {
      modelOption.selected = true;
    }
  }
  
  function getSelectedAIModel() {
    const modelsDropdown = document.getElementById('prefs-model-name');
    return modelsDropdown.value;
  }
  
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
  
  function updateAllCanonEntries(updateFunction) {
    // Create a transaction to batch all changes
    const tr = diagnostics.state.tr;
    let hasUpdates = false;
    
    // Traverse the document to find all canonEntry nodes
    diagnostics.state.doc.descendants((node, pos) => {
      if (node.type.name === 'canonEntry') {
        // Call the update function with the node and position
        const updatedAttrs = updateFunction(node);
        if (updatedAttrs) {
          // Apply the changes to the node at this position
          tr.setNodeMarkup(pos, null, {
            ...node.attrs,
            ...updatedAttrs
          });
          hasUpdates = true;
        }
      }
    });
    
    // Only dispatch if there were actual changes
    if (hasUpdates) {
      diagnostics.view.dispatch(tr);
    }
  }
  
  function addCanonEntry(entry) {
    let pos = diagnostics.state.selection.from + 2
    diagnostics.commands.setTextSelection(pos)
    diagnostics.commands.insertContent({
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
    pos = diagnostics.state.selection.from + 2
    diagnostics.commands.setTextSelection(pos)
    setTimeout(() => {
      diagnostics.view.dom.scrollIntoView({ behavior: 'smooth', block: 'end' });
    }, 0);
  }
  
  function setPreferencesUI(res) {
    console.log('Setting Preferences UI:', res);
    prefsMainPromptTextArea.textContent = res.main_prompt;
    prefsResponseLimitTextArea.textContent = res.response_limit;
    prefsFinalPreambleTextArea.textContent = res.final_preamble;
    prefsProseStyleTextArea.textContent = res.prose_style;
    prefsVibeModeContextTextArea.textContent = res.vibe_mode_context;
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
    prefsGameTimeSeconds.value = res.game_timer_ms / 1000;
    prefsGameTimeSecondsValue.textContent = res.game_timer_ms / 1000;
    setSelectedAIProvider(res.ai_provider);
    setSelectedAIModel(res.ai_model_name);
    prefsOllamaUrl.value = res.ollama_url;
    prefsLMStudioUrl.value = res.lm_studio_url;
    
    // Set the vibe genre radio button based on preferences
    setSelectedVibeGenre(res.vibe_mode_starter_genre_name);
    
    invoke("load_openai_api_key_from_keyring", {}).then((res) => {
      openaiApiKeyEl.value = res;
    });
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
  
  // Make updateAllCanonEntries available globally
  window.updateAllCanonEntries = updateAllCanonEntries;
  
  // Make the scrollEditorToSelection function available globally
  //window.scrollEditorToSelection = scrollEditorToSelection;
  // Function to get the selected vibe genre
  function getSelectedVibeGenre() {
    const selectedRadio = document.querySelector('input[name="vibe-genre"]:checked');
    return selectedRadio ? selectedRadio.value : null;
  }
  
  // Function to set the selected vibe genre
  function setSelectedVibeGenre(genreName) {
    const genreRadio = document.querySelector(`input[name="vibe-genre"][value="${genreName}"]`);
    if (genreRadio) {
      genreRadio.checked = true;
    }
  }
  
  // Function to update the vibe context textarea based on the selected genre
  function updateVibeContextFromGenre(genreName) {
    // This would typically involve a call to the backend to get the genre context
    // For now, we'll just update the textarea with the genre name
    // In a real implementation, you'd get the full context from the backend
    invoke("get_vibe_genre_context", { genreName }).then((context) => {
      prefsVibeModeContextTextArea.value = context;
    }).catch(error => {
      console.error("Error getting vibe genre context:", error);
    });
  }
  
  document.addEventListener("DOMContentLoaded", () => {
    // Get all vibe genre radio buttons and the context textarea
    const vibeGenreRadios = document.querySelectorAll('input[name="vibe-genre"]');
    const vibeContextTextarea = document.getElementById('prefs-vibe-mode-context');
    
    // Add event listeners to each vibe genre radio button
    vibeGenreRadios.forEach(radio => {
      radio.addEventListener('change', async (event) => {
        if (event.target.checked) {
          const selectedGenre = event.target.value;
          
          try {
            // Call the backend to get the context for this genre
            const context = await invoke('get_vibe_genre_context', { 
              genreName: selectedGenre 
            });
            
            // Update the textarea with the context from the backend
            vibeContextTextarea.value = context;
            
            console.log(`Updated vibe context for genre: ${selectedGenre}`);
          } catch (error) {
            console.error(`Failed to get context for genre ${selectedGenre}:`, error);
          }
        }
      });
    });
  });
  
  
  // Helper function to determine if we should force refresh
  function shouldForceRefresh(currentContext, previousContext) {
    addSimpleLogEntry({
      id: Date.now(),
      timestamp: Date.now(),
      message: '🔄 Checking if we should force refresh based on context changes:<br/>currentContext='+currentContext+ '<br/>previousContext='+previousContext,
      level: 'debug'
    });
    
    // First completion should always force refresh
    if (!previousContext) {
      addSimpleLogEntry({
        id: Date.now(),
        timestamp: Date.now(),
        message: '🟢 Force refresh: No previous context (first completion).',
        level: 'debug'
      });
      return true;
    }
    
    // Calculate how much the context has changed
    const lengthDiff = Math.abs(currentContext.length - previousContext.length);
    const percentChange = lengthDiff / Math.max(previousContext.length, 1);
    
    // Force refresh if:
    // 1. Context changed by more than 20%
    if (percentChange > 0.2) {
      addSimpleLogEntry({
        id: Date.now(),
        timestamp: Date.now(),
        message: `🟢 Force refresh: Context changed by more than 20% (${(percentChange*100).toFixed(1)}%).`,
        level: 'debug'
      });
      return true;
    }
    
    // 2. Added or removed more than 100 characters
    if (lengthDiff > 100) {
      addSimpleLogEntry({
        id: Date.now(),
        timestamp: Date.now(),
        message: `🟢 Force refresh: Length difference is more than 100 (${lengthDiff}).`,
        level: 'debug'
      });
      return true;
    }
    
    // 3. The context is entirely different (optional, more expensive check)
    let differentChars = 0;
    const minLength = Math.min(currentContext.length, previousContext.length);
    for (let i = 0; i < minLength; i++) {
      if (currentContext[i] !== previousContext[i]) {
        differentChars++;
      }
    }
    if (differentChars > minLength * 0.5) {
      addSimpleLogEntry({
        id: Date.now(),
        timestamp: Date.now(),
        message: `🟢 Force refresh: More than 50% of characters are different (${differentChars} of ${minLength}).`,
        level: 'debug'
      });
      return true;
    }
    
    // Otherwise, use cached results
    addSimpleLogEntry({
      id: Date.now(),
      timestamp: Date.now(),
      message: '🔵 No force refresh: Using cached results.',
      level: 'debug'
    });
    return false;
  }
  
  /**
  * Fetches a streaming completion from the backend using the current context and system message.
  *
  * @param {string} context - The user input or editor context to send to the backend.
  * @param {string} systemMessage - The system prompt or instructions for the LLM.
  * @param {AbortSignal} abortSignal - Signal to allow cancellation of the streaming request.
  * @param {boolean} [forceRefresh=false] - If true, forces a new RAG search instead of using cached results.
  * @param {boolean} withRagForStreaming - If true, enables retrieval-augmented generation (RAG) for the completion; if false, disables RAG.
  * @param {function} [onChunk=null] - Optional callback function that is called with each streamed chunk of text as it arrives.
  *                                    Use this to process or display partial completions in real time.
  * @returns {Promise<string>} - Resolves with the full concatenated completion text when streaming is finished.
  *
  * This function sets up an event listener for 'completion-chunk' events from the backend,
  * invokes the streaming completion command, and accumulates the streamed chunks.
  * The listener is cleaned up when the request completes or is aborted.
  */
  async function fetchStreamingCompletion(context, systemMessage, abortSignal, forceRefresh = false, withRagForStreaming, onChunk = null) {
    return new Promise(async (resolve, reject) => { // Added async here
      // Create a buffer to accumulate the response
      let fullResponse = '';
      
      // Set up the event listener to receive chunks
      // Fixed: await the Promise to get the actual unlisten function
      const unlisten = await listen('completion-chunk', (event) => {
        const chunk = event.payload;
        fullResponse += chunk;
        // Call your callback with each chunk
        if (onChunk) onChunk(chunk);
      });
      
      // Make the streaming request
      invoke('streaming_completion_from_context', {
        context: context,
        systemMessage: systemMessage,
        forceRefresh: forceRefresh,
        withRagForStreaming: withRagForStreaming
      })
      .then(() => {
        unlisten(); // Now this is a function
        resolve(fullResponse);
      })
      .catch((error) => {
        unlisten(); // Now this is a function
        reject(error);
      });
      
      // Handle abort signal
      abortSignal.addEventListener('abort', () => {
        if (unlisten) unlisten(); // Now this is a function
        reject(new Error('aborted'));
      });
    });
  }
  
  
  function setGhostSuggestion(suggestion) {
    const ext = editor.extensionManager.extensions.find(ext => ext.name === 'ghostCompletionDecoration')
    if (ext) {
      ext.options.suggestion = suggestion
      editor.view.updateState(editor.state) // force re-render
    }
  }
  
  async function* loadCompletionsStream(n = 5, withRagForStreaming = true, abortSignal) {
    addSimpleLogEntry({
      id: Date.now(),
      timestamp: Date.now(),
      message: '🔄 loadCompletionsStream started, requesting ' + n + ' completions',
      level: 'info'
    });
    
    for (let i = 0; i < n; i++) {
      if (abortSignal.aborted) {
        addSimpleLogEntry({
          id: Date.now(),
          timestamp: Date.now(),
          message: '⚠️ Aborting loadCompletionsStream at iteration ' + i,
          level: 'warn'
        });
        break;
      }
      
      const { state } = editor;
      const cursorPos = state.selection.from;
      const docText = editor.getText();
      const leftText = docText.slice(0, cursorPos);
      
      // Split into words and take the last 300 (or fewer)
      const words = leftText.split(/\s+/);
      const limitedWords = words.slice(-300);
      const currentContext = limitedWords.join(' ');
      
      const systemMessage = prefsMainPromptTextArea.value;
      
      addSimpleLogEntry({
        id: Date.now(),
        timestamp: Date.now(),
        message: '🔄 Fetching completion ' + (i + 1) + ' of ' + n,
        level: 'debug'
      });
      
      try {
        const forceRefresh = shouldForceRefresh(currentContext, lastTriggerContext);
        lastTriggerContext = currentContext;
        
        let streamedText = '';
        // Define the onChunk callback to handle streamed text
        // This will accumulate the streamed text and show ghost suggestions
        // for the first (active) completion
        const onChunk = (chunk) => {
          streamedText += chunk;
          // Only show ghost suggestion for the first (active) completion
          if (i === 0 && currentCompletionIndex === 0) {
            showGhostCompletion(editor, streamedText);
          }
        };
        // Call the fetchStreamingCompletion function with the onChunk callback
        addSimpleLogEntry({
          id: Date.now(),
          timestamp: Date.now(),
          message: '🔄 Calling fetchStreamingCompletion for completion ' + (i + 1) + ' with forceRefresh=' + forceRefresh,
          level: 'debug'
        });
        const result = await fetchStreamingCompletion(
          currentContext,
          systemMessage,
          abortSignal,
          forceRefresh,
          withRagForStreaming,
          onChunk
        );
        
        addSimpleLogEntry({
          id: Date.now(),
          timestamp: Date.now(),
          message: '✅ Completion ' + (i + 1) + ' received successfully with forceRefresh=' + forceRefresh,
          level: 'debug'
        });
        
        if (abortSignal.aborted) break;
        yield result;
      } catch (err) {
        addSimpleLogEntry({
          id: Date.now(),
          timestamp: Date.now(),
          message: '❌ Error fetching completion ' + (i + 1) + ': ' + err.message,
          level: 'error'
        });
        
        if (err.message !== 'aborted') {
          throw err; // Re-throw non-abort errors
        }
      }
    }
  }
  
  // 1. Load completions with optional RAG and streaming
  async function loadCompletions(n = 5, withRagForStreaming = true, loadMore = false, abortSignal = { aborted: false, addEventListener: () => {} }) {
    if (!loadMore) {
      completions = [];
      currentCompletionIndex = 0;
    }
    for await (const result of loadCompletionsStream(n, withRagForStreaming, abortSignal)) {
      completions.push(result);
    }
    addSimpleLogEntry({
      id: Date.now(),
      timestamp: Date.now(),
      message: 'Showing current completion at index: ' + (completions[currentCompletionIndex] || '(Empty)'),
      level: 'debug'
    });
    return completions;
  }
  
  // 2. Add more logging to triggerCompletions function
  async function triggerCompletions() {
    
    updateVibeStatus('thinking', true);
    if (isRetrievingCompletions) {
      addSimpleLogEntry({
        id: Date.now(),
        timestamp: Date.now(),
        message: '⚠️ Skipping triggerCompletions because isRetrievingCompletions is true',
        level: 'warn'
      });
      return;
    }
    
    isRetrievingCompletions = true;
    userHasTypedSinceLastCompletion = false;
    completionAbortController = new AbortController();
    completions = [];
    currentCompletionIndex = 0;
    //let streamingNoRagButton = document.querySelector("#streaming-no-rag-mode-btn");
    const withRagForStreaming = document.querySelector("#streaming-mode-btn").classList.contains("button-overline") ? false : true;
    try {     
      for await (const result of loadCompletionsStream(5, withRagForStreaming,completionAbortController.signal)) {
        completions.push(result);
        // Only update ghost if we're at the last completion (i.e., user is viewing the latest)
        if (currentCompletionIndex === completions.length - 1) {
          updateGhostCompletion();
        }
      }
    } catch (err) {
      addSimpleLogEntry({
        id: Date.now(),
        timestamp: Date.now(),
        message: '❌ Error in loadCompletionsStream: ' + err.message,
        level: 'error'
      });
      
      completions = [];
      setGhostSuggestion('');
      currentCompletionIndex = 0;
      ghostStartPos = null;
    } finally {
      isRetrievingCompletions = false;
      completionAbortController = null;
      
      addSimpleLogEntry({
        id: Date.now(),
        timestamp: Date.now(),
        message: '✅ Completed triggerCompletions execution',
        level: 'debug'
      });
    }
    
    updateVibeStatus('writing');
  }
  
  
  
  // Accept the ghost completion
  function acceptGhostCompletion() {
    const suggestion = completions[currentCompletionIndex] || ''
    if (!suggestion) return
    const { state, view } = editor
    const { from } = state.selection
    const docText = editor.getText();
    updateVibeStatus('writing', false);
    // Only insert the part of the suggestion that hasn't been typed
    // ghostStartPos is where the suggestion started
    const alreadyTyped = docText.slice(ghostStartPos, from);
    let toInsert = suggestion;
    if (alreadyTyped && suggestion.startsWith(alreadyTyped)) {
      toInsert = suggestion.slice(alreadyTyped.length);
    }
    
    let tr = state.tr;
    tr = tr.insertText(toInsert, from);
    tr = tr.setSelection(state.selection.constructor.create(tr.doc, from + toInsert.length));
    view.dispatch(tr);
    
    setGhostSuggestion('');
    editor.commands.focus();
  }
  
  
  function showGhostCompletion(editor, suggestion) {
    updateVibeStatus('emanating');
    const ext = editor.extensionManager.extensions.find(ext => ext.name === 'ghostCompletionDecoration');
    if (ext) {
      ext.options.suggestion = suggestion;
      editor.view.updateState(editor.state);
      if (suggestion && ghostStartPos === null) {
        ghostStartPos = editor.state.selection.from;
      }
      if (!suggestion) {
        // Only clear ghostStartPos when suggestion is cleared
        ghostStartPos = null;
        updateVibeStatus('writing');
      }
    }
  }
  
  function updateGhostCompletion() {
    const suggestion = completions[currentCompletionIndex] || '';
    showGhostCompletion(editor, suggestion);
  }
  //let streamingNoRagButton; // Declare at the top
  
  window.addEventListener("DOMContentLoaded", async () => {
    //streamingNoRagButton = document.querySelector("#streaming-no-rag-mode-btn");
    
    // Cycle completions
    editor.view.dom.addEventListener('keydown', async (e) => {
      if (e.shiftKey && (e.key === 'ArrowUp' || e.key === 'ArrowDown') && completions.length > 0) {
        if (e.key === 'ArrowUp') {
          currentCompletionIndex = (currentCompletionIndex - 1 + completions.length) % completions.length;
          updateGhostCompletion();
          e.preventDefault();
          return;
        }
        
        if (e.key === 'ArrowDown') {
          // Only allow cycling if not currently loading
          if (isRetrievingCompletions) {
            e.preventDefault();
            return;
          }
          
          
          // If near the end, load more and wait for it to finish before updating ghost
          if (completions.length - currentCompletionIndex <= PRELOAD_THRESHOLD) {
            addSimpleLogEntry({
              id: Date.now(),
              timestamp: Date.now(),
              message: '🔄 Near end of completions {}, loading more...',
              level: 'debug'
            });
            
            updateVibeStatus('thinking', true);
            isRetrievingCompletions = true;
            const streamingButton = document.querySelector("#streaming-mode-btn");
            const withRagForStreaming = !streamingButton.classList.contains("button-overline");
            await loadCompletions(5, withRagForStreaming, true);
            isRetrievingCompletions = false;
            updateVibeStatus('writing');
          }
          
          currentCompletionIndex = (currentCompletionIndex + 1) % completions.length;
          
          // Only update ghost after completions are loaded
          updateGhostCompletion();
          e.preventDefault();
          return;
        }
      }
      
      
      // Cycle with Shift+Up/Down
      // if (e.shiftKey && (e.key === 'ArrowUp' || e.key === 'ArrowDown')  && completions.length > 0) {
      //   if (e.key === 'ArrowUp') {
      //     currentCompletionIndex = (currentCompletionIndex - 1 + completions.length) % completions.length
      //   } else if (e.key === 'ArrowDown') {
      //     currentCompletionIndex = (currentCompletionIndex + 1) % completions.length
      //     // If near the end, load more
      //     if ( completions.length - currentCompletionIndex <= PRELOAD_THRESHOLD ) {
      //       //if (currentCompletionIndex === 0 || completions[currentCompletionIndex] === undefined) {
      //       updateGhostCompletion()
      //       await loadCompletions(3, true);
      //       //}
      //     }
      //   }
      //   updateGhostCompletion()
      //   e.preventDefault()
      // }
      
      // Accept with Tab
      if (e.key === 'Tab') {
        // Only accept if there's an active ghost suggestion
        if (completions.length > 0 && ghostStartPos !== null) {
          userHasTypedSinceLastCompletion = false;
          acceptGhostCompletion()
          e.preventDefault()
        }
      }
    })
    
    
    editor.on('update', ({ editor }) => {
      const suggestion = completions[currentCompletionIndex] || '';
      const { from } = editor.state.selection;
      const docText = editor.getText();
      
      if (ghostStartPos === null) {
        // addSimpleLogEntry({
        //   id: Date.now(),
        //   timestamp: Date.now(),
        //   message: 'Ghost start position is null, skipping ghost suggestion update.',
        //   level: 'debug'
        // });
        return;
      }
      
      // Try both slicing methods to see what's actually happening
      const userTyped = docText.slice(ghostStartPos-1, from);
      const userTypedWithout = docText.slice(ghostStartPos, from);
      
      // Log to diagnostics so we can see what's happening
      addSimpleLogEntry({
        id: "typeahead-debug-" + Date.now(),
        timestamp: Date.now(),
        message: `Typeahead debug:
    - ghostStartPos: ${ghostStartPos}
    - from: ${from}
    - userTyped (with -1): "${userTyped}"
    - userTyped (without -1): "${userTypedWithout}"
    - suggestion: "${suggestion}"
    - startsWith (with -1): ${suggestion.startsWith(userTyped)}
    - startsWith (without -1): ${suggestion.startsWith(userTypedWithout)}`,
        level: 'debug'
      });
      // // Debug log
      //     console.log({
      //       ghostStartPos,
      //       from,
      //       userTyped,
      //       suggestion,
      //       suggestionStartsWithUserTyped: suggestion.startsWith(userTyped)
      //     });
      
      if (userTyped.length === 0) {
        setGhostSuggestion(suggestion);
        return;
      }
      
      if (suggestion.startsWith(userTyped)) {
        // User is typing the suggestion correctly: shrink the ghost
        setGhostSuggestion(suggestion.slice(userTyped.length));
        if (userTyped === suggestion) {
          // User finished typing the suggestion: clear the ghost
          setGhostSuggestion('');
          completions = [];
          currentCompletionIndex = 0;
          ghostStartPos = null;
        }
      } else {
        // User mistyped: clear completions, ghost, and reset indices
        setGhostSuggestion('');
        completions = [];
        currentCompletionIndex = 0;
        ghostStartPos = null;
        userHasTypedSinceLastCompletion = true;
      }
    });
  });
  
  // Tour function
  function startAppTour() {
    const tour = new Shepherd.Tour({
      defaultStepOptions: {
        scrollTo: true,
        cancelIcon: { enabled: true },
        classes: 'shadow-lg bg-white rounded',
        modalOverlayOpeningPadding: 8,
        modalOverlayOpeningRadius: 6
      }
    });

    tour.addStep({
      id: 'menu-btn',
      text: 'This is the menu button. Click here to open the main menu.',
      attachTo: { element: '#panel-toggle-btn', on: 'bottom' },
      buttons: [
        { text: 'Next', action: tour.next }
      ]
    });

    tour.addStep({
      id: 'editor',
      text: 'This is your main editor. Type your content here.',
      attachTo: { element: '#main-editor', on: 'top' },
      buttons: [
        { text: 'Back', action: tour.back },
        { text: 'Next', action: tour.next }
      ]
    });

    tour.addStep({
      id: 'simplify-btn',
      text: 'Use this button to simplify your text using AI.',
      attachTo: { element: '#simplify-btn', on: 'bottom' },
      buttons: [
        { text: 'Back', action: tour.back },
        { text: 'Done', action: tour.complete }
      ]
    });

    tour.start();
  }