let completions = [];
  let currentCompletionIndex = 0;
  let loadingMore = false;
  let ghostActive = false;
  
  
  async function fetchStreamingCompletion(context, systemMessage) {
    return new Promise((resolve, reject) => {
      let completion = '';
      let unlisten = null;
      
      // Listen for streaming chunks
      window.__TAURI__.event.listen('completion-chunk', (event) => {
        completion += event.payload;
      }).then((unlistenFn) => {
        unlisten = unlistenFn;
      });
      
      // Call the backend
      window.__TAURI__.core.invoke('streaming_completion_from_context', {
        context,
        systemMessage,
      }).then(() => {
        if (unlisten) unlisten();
        resolve(completion);
      }).catch((err) => {
        if (unlisten) unlisten();
        reject(err);
      });
    });
  }

  function setGhostSuggestion(suggestion) {
  const ext = editor.extensionManager.extensions.find(ext => ext.name === 'ghostCompletionDecoration')
  if (ext) {
    ext.options.suggestion = suggestion
    editor.view.updateState(editor.state) // force re-render
    ghostActive = !!suggestion
  }
}

function updateGhostCompletion() {
  const suggestion = completions[currentCompletionIndex] || ''
  setGhostSuggestion(suggestion)
}
  async function loadCompletions(n = 3) {
    loadingMore = true;
    completions = [];
    currentCompletionIndex = 0;
    for (let i = 0; i < n; i++) {
      // Use your editor's current text and a system message
      const context = editor.getText();
      const systemMessage = "Provide a very short text to complete the passage."; //prefsMainPromptTextArea.value;
      const result = await fetchStreamingCompletion(context, systemMessage);
      completions.push(result);
    }
    loadingMore = false;
    showCurrentCompletion();
    return completions;
  }
  
  function showCurrentCompletion() {
    
    // const preview = document.getElementById('completion-preview');
    // if (completions.length === 0) {
    //   preview.textContent = '(No completions loaded)';
    //   return;
    // }
    // preview.textContent = completions[currentCompletionIndex] || '(Empty)';
  }
  
  // Show the current ghost completion
  function updateGhostCompletion() {
    console.log('Completions is:', completions);
    const suggestion = completions[currentCompletionIndex] || ''
    showGhostCompletion(editor, suggestion)
    ghostActive = !!suggestion
  }
  
  
  
  // Accept the ghost completion
  function acceptGhostCompletion() {
  const suggestion = completions[currentCompletionIndex] || ''
  if (!suggestion) return
  const { state, view } = editor
  const { from } = state.selection
  let tr = state.tr
  tr = tr.insertText(suggestion, from)
  tr = tr.setSelection(state.selection.constructor.create(tr.doc, from + suggestion.length))
  view.dispatch(tr)
  setGhostSuggestion('')
  editor.commands.focus()
  ghostActive = false
}
    
  window.addEventListener("DOMContentLoaded", async () => {
    
    // Cycle completions
    document.addEventListener('keydown', async (e) => {
      if (!ghostActive) return
      
      // Cycle with Shift+Up/Down
      if (e.shiftKey && (e.key === 'ArrowUp' || e.key === 'ArrowDown')) {
        if (e.key === 'ArrowUp') {
          currentCompletionIndex = (currentCompletionIndex - 1 + completions.length) % completions.length
        } else if (e.key === 'ArrowDown') {
          currentCompletionIndex = (currentCompletionIndex + 1) % completions.length
        }
        updateGhostCompletion()
        e.preventDefault()
      }
      
      // Accept with Tab
      if (e.key === 'Tab') {
        acceptGhostCompletion()
        e.preventDefault()
      }
    })

    editor.on('update', ({ editor }) => {
  if (!ghostActive) return
  const suggestion = completions[currentCompletionIndex] || ''
  const { from } = editor.state.selection
  const docText = editor.getText()
  // Get the text the user has typed at the cursor
  const before = docText.slice(0, from)
  const after = docText.slice(from)
  // Find the last word or chars the user typed
  // We'll use the last N chars where N = suggestion.length
  const userTyped = before.slice(-suggestion.length)
  if (suggestion.startsWith(userTyped) && userTyped.length > 0) {
    setGhostSuggestion(suggestion.slice(userTyped.length))
    if (userTyped === suggestion) setGhostSuggestion('')
  } else if (userTyped.length > 0) {
    setGhostSuggestion('')
  }
})
    
    document.getElementById('test-streaming-btn').addEventListener('click', async () => {
      triggerCompletions();
    });
    
    // Keyboard navigation
    document.addEventListener('keydown', async (e) => {
      if (!completions.length || loadingMore) return;
      if (e.shiftKey && (e.key === 'ArrowUp' || e.key === 'ArrowDown')) {
        if (e.key === 'ArrowUp') {
          currentCompletionIndex = (currentCompletionIndex - 1 + completions.length) % completions.length;
        } else if (e.key === 'ArrowDown') {
          currentCompletionIndex = (currentCompletionIndex + 1) % completions.length;
          // If at end, load more
          if (currentCompletionIndex === 0) {
            await loadCompletions(3);
          }
        }
        showCurrentCompletion();
        e.preventDefault();
      }
    });
  });
  
  
  
  
  
  async function triggerCompletions() {
    // Fetch completions as before
    completions = await loadCompletions(3)
    currentCompletionIndex = 0
    console.log('Loaded completions:', completions)
    updateGhostCompletion()
  }
  
  function removeAllGhostCompletions(editor) {
    const { state, view } = editor;
    let tr = state.tr;
    let found = false;
    
    state.doc.descendants((node, pos) => {
      if (node.isText && node.marks.some(mark => mark.type.name === 'ghostCompletionMark')) {
        tr = tr.delete(pos, pos + node.nodeSize);
        found = true;
      }
    });
    
    if (found) {
      view.dispatch(tr);
    }
  }

  function showGhostCompletion(editor, suggestion) {
    removeAllGhostCompletions(editor); // Always clear old ghost text
    
    const { state, view } = editor;
    const { from } = state.selection;
    let tr = state.tr;
    
  }