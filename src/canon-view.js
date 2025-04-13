// Import Tauri v2 APIs
import { listen } from '@tauri-apps/api/event';
import { openPath, revealItemInDir } from '@tauri-apps/plugin-opener';
const { invoke } = window.__TAURI__.core;

// Store the current documents
let allDocuments = [];
let selectedDocumentId = null;
// Track the unique model names and currently selected model filter
let uniqueModels = [];
let currentModelFilter = null;

/**
 * Open the file in the default application
 */
async function openFileInDefaultApp(filePath) {
  try {
    //await openUrl('https://github.com/tauri-apps/tauri');
    await openPath(filePath);
  } catch (error) {
    console.error('Failed to open file:', error);
    alert(`Failed to open file: ${error}`);
  }
}

/**
 * Open the folder containing the file
 */
async function openContainingFolder(filePath) {
  try {
    // Get the directory path by removing the filename
    //const dirPath = filePath.substring(0, filePath.lastIndexOf('/')+1);
    await revealItemInDir(filePath);
  } catch (error) {
    console.error('Failed to open folder:', error);
    alert(`Failed to open folder: ${error}`);
  }
}

/**
 * Creates model filter buttons based on unique models in the document list
 */
function createModelFilterButtons() {
  const modelButtonsContainer = document.getElementById('model-canon-buttons');
  
  // Clear existing buttons
  modelButtonsContainer.innerHTML = '';
  
  // Create a button for each unique model
  uniqueModels.forEach(model => {
    const button = document.createElement('button');
    button.textContent = model.toUpperCase();
    button.classList.add(
      'canon-entry-button',
      'enabled',
      'text-xs'
    );
    
    // If this model is the current filter, add visual indication
    if (currentModelFilter === model) {
      button.classList.add('button-in');
    }
    
    // Add click event to filter documents
    button.addEventListener('click', () => {
      // Toggle filter - if already selected, turn off filter
      if (currentModelFilter === model) {
        currentModelFilter = null;
        button.classList.remove('button-in');
      } else {
        // Remove button-in class from all buttons
        document.querySelectorAll('#model-canon-buttons button').forEach(btn => {
          btn.classList.remove('button-in');
        });
        
        // Set this model as the filter and add visual indication
        currentModelFilter = model;
        button.classList.add('button-in');
      }
      
      // Refresh document list with the filter
      refreshDocumentList();
    });
    
    modelButtonsContainer.appendChild(button);
  });
}

/**
 * Extract unique model names from all documents
 */
function extractUniqueModels(documents) {
  // Get all unique embedding model names
  const models = new Set(documents.map(doc => doc.embedding_model_name));
  uniqueModels = Array.from(models);
  
  // Create buttons for these models
  createModelFilterButtons();
}

/**
 * Refresh the document list based on current filters
 */
function refreshDocumentList() {
  // Make a copy of all documents
  let filteredDocs = [...allDocuments];
  
  // Apply model filter if active
  if (currentModelFilter) {
    filteredDocs = filteredDocs.filter(doc => 
      doc.embedding_model_name === currentModelFilter
    );
  }
  
  // Repopulate with filtered documents
  renderDocumentList(filteredDocs);
}

/**
 * Render the document list based on provided documents
 */
function renderDocumentList(documents) {
  const container = document.getElementById('document-items-container');
  container.innerHTML = '';
  
  // Sort documents based on the selected sort option
  const sortSelect = document.getElementById('sort-select');
  documents = sortDocuments(documents, sortSelect.value);

  // Render each document in the list
  documents.forEach(doc => {
    const itemDiv = document.createElement('div');
    itemDiv.classList.add(
      'document-item', 
      'p-2', 
      'border-b', 
      'border-gray-200', 
      'hover:bg-gray-900', 
      'flex', 
      'flex-col',
      'cursor-pointer',
      'gap-2', // Add gap between rows
      'py-3', // Increase vertical padding
      'min-h-[5em]', // Ensure minimum height
    );
    itemDiv.dataset.docId = doc.id;
    
    // If the document is paused, add visual indication
    if (doc.paused) {
      itemDiv.classList.add('bg-gray-600', '!text-gray-300');
    }
    
    // When clicking on the item, show the document details
    itemDiv.onclick = () => selectDocument(doc.id);

    // First row: Document name
    const nameRow = document.createElement('div');
    nameRow.classList.add('w-full');
    
    // Document name
    const nameSpan = document.createElement('span');
    nameSpan.classList.add(
'flex-grow', 
      'truncate',
      'text-wrap', 
'font-[InputMonoCondensed]', 
      'font-medium',
      'text-sm',
      '!leading-tight', // Add explicit line-height
      'block', // Make it a block element to respect line height
      'pb-1' // Add some padding at the bottom
);
    nameSpan.textContent = doc.name;
    nameRow.appendChild(nameSpan);
    
    // Second row: Authors (always show the row, even if empty)
    const authorsRow = document.createElement('div');
    authorsRow.classList.add('text-xs', 'text-gray-400', 'font-[InputMonoCondensed]');
    
    if (doc.authors && doc.authors.length > 0) {
      const authorsLabel = document.createElement('span');
      authorsLabel.classList.add('font-medium', 'mr-1');
      authorsLabel.textContent = 'Authors:';
      authorsRow.appendChild(authorsLabel);
      
      const authorsText = document.createElement('span');
      authorsText.textContent = doc.authors.join(', ');
      authorsRow.appendChild(authorsText);
    } else {
      authorsRow.textContent = 'No authors';
      authorsRow.classList.add('italic');
    }
    
    // Third row: Buttons
    const btnRow = document.createElement('div');
    btnRow.classList.add('flex', 'justify-between', 'items-center', 'w-full', 'mt-1');
    
    // Model name (left side of button row)
    const modelSpan = document.createElement('span');
    modelSpan.classList.add('text-xs', 'text-gray-500');
    modelSpan.textContent = doc.embedding_model_name;
    btnRow.appendChild(modelSpan);
    
    // Button container (right side of button row)
    const btnContainer = document.createElement('div');
    btnContainer.classList.add('flex', 'gap-1');
    
    // Pause/Resume button
    const pauseBtn = document.createElement('button');
    pauseBtn.textContent = doc.paused ? 'RESUME' : 'PAUSE';
    pauseBtn.classList.add(
      'canon-entry-button', 
      'text-xs',
      'enabled',
    );
    if (doc.paused) {
      pauseBtn.classList.add('pause', 'button-in', '!bg-amber-300', '!text-gray-600');
    } else {
      pauseBtn.classList.remove('pause', 'button-in');
      pauseBtn.classList.add('button-out');
    }   
    pauseBtn.onclick = (e) => {
      e.stopPropagation(); // Prevent triggering the item click
      togglePauseDocument(doc.id, !doc.paused);
    };
    
    // Delete button
    const delBtn = document.createElement('button');
    delBtn.textContent = 'DELETE';
    delBtn.classList.add(
      'canon-entry-button', 'enabled', 'del', 'text-xs'
    );
    delBtn.onclick = (e) => {
      e.stopPropagation(); // Prevent triggering the item click
      if (confirm(`Are you sure you want to delete "${doc.name}"?`)) {
        deleteDocument(doc.id);
      }
    };
    
    btnContainer.appendChild(pauseBtn);
    btnContainer.appendChild(delBtn);
    btnRow.appendChild(btnContainer);
    
    // Append all rows to the item
    itemDiv.appendChild(nameRow);
    itemDiv.appendChild(authorsRow);
    itemDiv.appendChild(btnRow);
    
    container.appendChild(itemDiv);
  });
}

/**
 * Populate the document list with the provided documents
 */
function populateDocumentList(documents) {
  allDocuments = documents;

  // Extract unique models and create filter buttons
  extractUniqueModels(documents);
  
  // Render document list (applying any active filters)
  refreshDocumentList();
}

/**
 * Sort documents by the specified field
 */
function sortDocuments(documents, sortBy) {
  const sortedDocs = [...documents]; // Make a copy to avoid modifying original array
  
  switch (sortBy) {
    case 'name':
      sortedDocs.sort((a, b) => a.name.localeCompare(b.name));
      break;
    case 'date':
      sortedDocs.sort((a, b) => new Date(b.created_at) - new Date(a.created_at));
      break;
    case 'model':
      sortedDocs.sort((a, b) => a.embedding_model_name.localeCompare(b.embedding_model_name));
      break;
    case 'authors':
      sortedDocs.sort((a, b) => {
        const aAuthors = (a.authors || []).join(', ');
        const bAuthors = (b.authors || []).join(', ');
        return aAuthors.localeCompare(bAuthors);
    });
    break;
  }
  
  return sortedDocs;
}

/**
 * Select a document and display its details
 */
function selectDocument(docId) {
  // Update the selected state
  selectedDocumentId = docId;
  
  // Clear previous selection visual indicator
  document.querySelectorAll('.document-item').forEach(item => {
    item.classList.remove('bg-gray-600', 'border-l-[5px]', 'border-l-green-500');
  });
  
  // Add visual indicator to selected item
  const selectedItem = document.querySelector(`.document-item[data-doc-id="${docId}"]`);
  if (selectedItem) {
    selectedItem.classList.add('bg-gray-600', 'border-l-[5px]', 'border-l-green-500');
  }
  
  // Find the document
  const doc = allDocuments.find(d => d.id === docId);
  if (!doc) return;
  
  // Update the detail view
  document.getElementById('no-selection').classList.add('hidden');
  document.getElementById('document-details').classList.remove('hidden');
  
  document.getElementById('detail-name').value = doc.name;
  document.getElementById('detail-file-path').textContent = doc.file_path;
  document.getElementById('detail-date').textContent = formatDate(doc.created_at);
  document.getElementById('detail-model').textContent = doc.embedding_model_name;
  document.getElementById('detail-notes').value = doc.notes || '';
  
  // Clear and populate authors
  const authorsList = document.getElementById('authors-list');
  authorsList.innerHTML = '';
  const authors = doc.authors || [];
  
  renderAuthorTags(authors);
}

/**
 * Render author tags in the UI
 */
function renderAuthorTags(authors) {
  const authorsList = document.getElementById('authors-list');
  authorsList.innerHTML = '';
  
  authors.forEach(author => {
    if (!author.trim()) return; // Skip empty authors
    
    const tag = document.createElement('div');
    tag.classList.add('bg-gray-300', 'border-1', 'border-black', 'text-black', 'px-2', 'py-1', 'rounded', 'flex', 'items-center');
    
    const authorText = document.createElement('span');
    authorText.textContent = author;
    tag.appendChild(authorText);
    
    const removeBtn = document.createElement('button');
    removeBtn.textContent = '×';
    removeBtn.classList.add('ml-1', 'text-gray-500', 'hover:text-blue-700', 'font-bold');
    removeBtn.onclick = () => {
      // Get the currently selected document
      const doc = allDocuments.find(d => d.id === selectedDocumentId);
      if (doc) {
        // Remove this author from the array
        doc.authors = (doc.authors || []).filter(a => a !== author);
        // Re-render the tags
        renderAuthorTags(doc.authors);
      }
    };
    tag.appendChild(removeBtn);
    
    authorsList.appendChild(tag);
  });
}

/**
 * Add a new author to the selected document
 */
function addAuthor(authorInput) {
  // Get input value and clear the input
  const value = authorInput.value.trim();
  authorInput.value = '';
  
  if (!value) return; // Don't add empty authors
  
  // Get the currently selected document
  const doc = allDocuments.find(d => d.id === selectedDocumentId);
  if (!doc) return;
  
  // Initialize authors array if it doesn't exist
  if (!doc.authors) doc.authors = [];
  
  // Split by commas and add each author
  const newAuthors = value.split(',').map(a => a.trim()).filter(a => a);
  
  // Add authors that don't already exist
  newAuthors.forEach(author => {
    if (!doc.authors.includes(author)) {
      doc.authors.push(author);
    }
  });
  
  // Re-render the tags
  renderAuthorTags(doc.authors);
}

/**
 * Format a date string for display
 */
function formatDate(dateStr) {
  try {
    const date = new Date(dateStr);
    return date.toLocaleString();
  } catch (e) {
    return dateStr;
  }
}

/**
 * Toggle the pause state of a document
 */
async function togglePauseDocument(docId, pauseState) {
  try {
    await invoke('toggle_rag_pause', { id: docId.toString(), paused: pauseState });
    
    // Update local data
    const docIndex = allDocuments.findIndex(d => d.id === docId);
    if (docIndex >= 0) {
      allDocuments[docIndex].paused = pauseState;
      
      // Refresh the document list to reflect changes
      populateDocumentList(allDocuments);
      
      // If this was the selected document, reselect it
      if (selectedDocumentId === docId) {
        selectDocument(docId);
      }
    }
  } catch (error) {
    console.error('Failed to toggle pause state:', error);
    alert('Failed to update document state');
  }
}

/**
 * Set the pause state for all documents
 */
async function setAllPause(pauseState) {
  try {
    // Create an array of promises to update all documents
    const promises = allDocuments.map(doc => 
      invoke('toggle_rag_pause', { id: doc.id.toString(), paused: pauseState })
    );
    
    // Wait for all updates to complete
    await Promise.all(promises);
    
    // Update local data
    allDocuments.forEach(doc => {
      doc.paused = pauseState;
    });
    
    // Refresh the document list to reflect changes
    populateDocumentList(allDocuments);
    
    // If there was a selected document, reselect it to update the view
    if (selectedDocumentId) {
      selectDocument(selectedDocumentId);
    }
  } catch (error) {
    console.error('Failed to set all pause states:', error);
    alert('Failed to update document states');
  }
}

/**
 * Toggle the pause state of all documents
 */
async function toggleAllPause() {
  try {
    // Create an array of promises to update each document with its opposite state
    const promises = allDocuments.map(doc => 
      invoke('toggle_rag_pause', { id: doc.id.toString(), paused: !doc.paused })
    );
    
    // Wait for all updates to complete
    await Promise.all(promises);
    
    // Update local data by flipping each document's paused state
    allDocuments.forEach(doc => {
      doc.paused = !doc.paused;
    });
    
    // Refresh the document list to reflect changes
    populateDocumentList(allDocuments);
    
    // If there was a selected document, reselect it to update the view
    if (selectedDocumentId) {
      selectDocument(selectedDocumentId);
    }
  } catch (error) {
    console.error('Failed to toggle all pause states:', error);
    alert('Failed to update document states');
  }
}

/**
 * Delete a document
 */
async function deleteDocument(docId) {
  try {
    await invoke('delete_canon_entry', { docid: docId.toString() });
    
    // Update local data
    allDocuments = allDocuments.filter(d => d.id !== docId);
    
    // Refresh the document list
    populateDocumentList(allDocuments);
    
    // If the deleted document was selected, clear the details
    if (selectedDocumentId === docId) {
      selectedDocumentId = null;
      document.getElementById('document-details').classList.add('hidden');
      document.getElementById('no-selection').classList.remove('hidden');
    }
  } catch (error) {
    console.error('Failed to delete document:', error);
    alert('Failed to delete document');
  }
}

/**
 * Save changes to document details
 */
async function saveDocumentChanges() {
  if (!selectedDocumentId) return;
  
  const newName = document.getElementById('detail-name').value.trim();
  const newNotes = document.getElementById('detail-notes').value.trim();
  
  if (!newName) {
    alert('Document name cannot be empty');
    return;
  }
  
  // Get the currently selected document
  const doc = allDocuments.find(d => d.id === selectedDocumentId);
  if (!doc) return;
  
  try {
    // Call backend to update document
    await invoke('update_document_details', { 
      docId: selectedDocumentId.toString(),
      name: newName,
      notes: newNotes,
      authors: doc.authors || []
    });
    
    // Update local data
    const docIndex = allDocuments.findIndex(d => d.id === selectedDocumentId);
    if (docIndex >= 0) {
      allDocuments[docIndex].name = newName;
      allDocuments[docIndex].notes = newNotes;
      // authors are already updated in the object
      
      // Refresh the document list to reflect changes
      populateDocumentList(allDocuments);
      
      // Keep the current document selected
      selectDocument(selectedDocumentId);
    }
    
    alert('Changes saved successfully');
  } catch (error) {
    console.error('Failed to save document changes:', error);
    alert('Failed to save changes');
  }
}

// Initialize when DOM is loaded
window.addEventListener('DOMContentLoaded', async () => {
  // Set up event listener for sorting
  document.getElementById('sort-select').addEventListener('change', (e) => {
    refreshDocumentList();
    
    // Reselect the current document if there was one
    if (selectedDocumentId) {
      selectDocument(selectedDocumentId);
    }
  });
  
  // Set up event listeners for file and folder buttons
  document.getElementById('open-file-btn').addEventListener('click', () => {
    const doc = allDocuments.find(d => d.id === selectedDocumentId);
    if (doc && doc.file_path) {
      openFileInDefaultApp(doc.file_path);
    }
  });
  
  document.getElementById('open-folder-btn').addEventListener('click', () => {
    const doc = allDocuments.find(d => d.id === selectedDocumentId);
    if (doc && doc.file_path) {
      openContainingFolder(doc.file_path);
    }
  });
  
  // Set up event listener for save button
  document.getElementById('detail-save-btn').addEventListener('click', saveDocumentChanges);
  
  // Set up event listener for add author button
  const authorInput = document.getElementById('detail-authors');
  const addAuthorBtn = document.getElementById('add-author-btn');
  
  addAuthorBtn.addEventListener('click', () => {
    addAuthor(authorInput);
  });
  
  // Also add author when pressing Enter in the input
  authorInput.addEventListener('keypress', (e) => {
    if (e.key === 'Enter') {
      addAuthor(authorInput);
      e.preventDefault();
    }
  });
  
  // Set up event listeners for the batch document control buttons
  document.getElementById('all-off').addEventListener('click', () => {
    setAllPause(true); // Pause all documents
  });
  
  document.getElementById('all-on').addEventListener('click', () => {
    setAllPause(false); // Unpause all documents
  });
  
  document.getElementById('toggle-pause').addEventListener('click', toggleAllPause);
  
  // Listen for document list updates from the backend
  await listen('canon-list-to-control-panel', (event) => {
    try {
      const listing = JSON.parse(event.payload);
      populateDocumentList(listing.documents);
    } catch (error) {
      console.error('Error parsing document list:', error);
    }
  });

  // Request initial document list
  try {
    await invoke('list_canon_docs_to_canon_control_panel');
  } catch (error) {
    console.error('Failed to request document list:', error);
  }
});
