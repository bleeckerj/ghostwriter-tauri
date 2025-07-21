const { invoke } = window.__TAURI__.core;
/**
 * Simplify Text Module
 * Handles the simplification of text in the editor with LLM-based alternatives
 */

let alternatives = [];
let currentAlternativeIndex = 0;
let originalText = '';
let isSimplifying = false;

/**
 * Initialize the simplify functionality
 * @param {Object} editor - The Tiptap editor instance
 */
export function initSimplify(editor) {
  // Set up button click handler
  const simplifyBtn = document.getElementById('simplify-btn');
  if (simplifyBtn) {
    simplifyBtn.classList.remove('disabled');
    simplifyBtn.classList.add('enabled');
    
    simplifyBtn.addEventListener('click', () => {
      handleSimplifyButtonClick(editor);
    });
  }

  // Set up keyboard shortcuts for cycling through alternatives
  document.addEventListener('keydown', (event) => {
    // Option + Up Arrow to go to previous alternative
    if (event.altKey && event.key === 'ArrowUp' && alternatives.length > 0) {
      event.preventDefault();
      showPreviousAlternative(editor);
    }
    
    // Option + Down Arrow to go to next alternative
    if (event.altKey && event.key === 'ArrowDown' && alternatives.length > 0) {
      event.preventDefault();
      showNextAlternative(editor);
    }
    
    // Escape key to revert to original
    if (event.key === 'Escape' && alternatives.length > 0) {
      event.preventDefault();
      revertToOriginal(editor);
    }
  });
}

/**
 * Handle the click on the simplify button
 * @param {Object} editor - The Tiptap editor instance
 */
async function handleSimplifyButtonClick(editor) {
  if (isSimplifying) return; // Prevent multiple simultaneous simplifications
  
  isSimplifying = true;
  const simplifyBtn = document.getElementById('simplify-btn');
  simplifyBtn.classList.add('processing');
  
  try {
    // Get selected text or all text
    const { text, isSelection } = getTextToSimplify(editor);
    
    if (!text || text.trim() === '') {
      showNotification('No text to simplify');
      return;
    }
    
    // Save original for potential reversion
    originalText = text;
    
    // Call the backend with default parameters
    // Grade level 6, 3 alternatives
    const result = await invoke('simplify_text', {
      text: text,
      gradeLevel: 6,
      numAlternatives: 3
    });
    
    // Store alternatives and show the first one
    alternatives = result;
    currentAlternativeIndex = 0;
    
    if (alternatives.length > 0) {
      applyAlternativeToEditor(editor, alternatives[0], isSelection);
      showNotification(`Alternative 1/${alternatives.length} (Option+Up/Down to cycle, Esc to revert)`);
    } else {
      showNotification('No simplification alternatives generated');
    }
  } catch (error) {
    console.error('Simplification error:', error);
    showNotification(`Error: ${error.toString()}`);
  } finally {
    isSimplifying = false;
    simplifyBtn.classList.remove('processing');
  }
}

/**
 * Gets the text to be simplified - either selected text or entire editor content
 * @param {Object} editor - The Tiptap editor instance
 * @returns {Object} - The text and whether it's a selection
 */
function getTextToSimplify(editor) {
  const selection = editor.state.selection;
  
  // Check if there is a text selection
  if (selection && !selection.empty) {
    // Get the selected text
    const selectedText = editor.state.doc.textBetween(
      selection.from, 
      selection.to,
      ' ', // Block separator
      ' '  // Leaf separator
    );
    
    return { 
      text: selectedText,
      isSelection: true
    };
  }
  
  // No selection, get the entire document text
  const fullText = editor.state.doc.textContent;
  return { 
    text: fullText,
    isSelection: false
  };
}

/**
 * Apply an alternative to the editor
 * @param {Object} editor - The Tiptap editor instance
 * @param {string} alternativeText - The text to apply
 * @param {boolean} isSelection - Whether to replace only the selection
 */
function applyAlternativeToEditor(editor, alternativeText, isSelection) {
  // Split on \r\n or \n or \r
  const lines = alternativeText.split(/\r?\n/).map(line => line.trim()).filter(line => line.length > 0);

  // Build a document fragment with each line as a paragraph
  const content = lines.map(line => ({ type: 'paragraph', content: [{ type: 'text', text: line }] }));

  if (isSelection) {
    // Replace just the selected text
    editor.commands.deleteSelection();
    editor.commands.insertContent(content);
  } else {
    // Replace the entire document
    editor.commands.setContent(content);
  }
}

/**
 * Show the next alternative in the list
 * @param {Object} editor - The Tiptap editor instance
 */
function showNextAlternative(editor) {
  if (alternatives.length === 0) return;
  
  currentAlternativeIndex = (currentAlternativeIndex + 1) % alternatives.length;
  applyAlternativeToEditor(editor, alternatives[currentAlternativeIndex], 
    editor.state.selection && !editor.state.selection.empty);
  
  showNotification(`Alternative ${currentAlternativeIndex + 1}/${alternatives.length} (Option+Up/Down to cycle, Esc to revert)`);
}

/**
 * Show the previous alternative in the list
 * @param {Object} editor - The Tiptap editor instance
 */
function showPreviousAlternative(editor) {
  if (alternatives.length === 0) return;
  
  currentAlternativeIndex = (currentAlternativeIndex - 1 + alternatives.length) % alternatives.length;
  applyAlternativeToEditor(editor, alternatives[currentAlternativeIndex], 
    editor.state.selection && !editor.state.selection.empty);
  
  showNotification(`Alternative ${currentAlternativeIndex + 1}/${alternatives.length} (Option+Up/Down to cycle, Esc to revert)`);
}

/**
 * Revert to the original text
 * @param {Object} editor - The Tiptap editor instance
 */
function revertToOriginal(editor) {
  if (!originalText) return;
  
  applyAlternativeToEditor(editor, originalText, 
    editor.state.selection && !editor.state.selection.empty);
  
  // Clear alternatives
  alternatives = [];
  originalText = '';
  currentAlternativeIndex = 0;
  
  showNotification('Reverted to original text');
}

/**
 * Show a notification to the user
 * @param {string} message - The message to show
 */
function showNotification(message) {
  // Use the existing notification system if available
  if (window.showSimpleLogMessage) {
    window.showSimpleLogMessage(message, '', 'info');
    return;
  }
  
  // Fallback notification
  console.info(message);
}

// Export functions that might be needed elsewhere
export {
  handleSimplifyButtonClick,
  revertToOriginal
};