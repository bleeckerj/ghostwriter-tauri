import { listen } from '@tauri-apps/api/event';
const { invoke } = window.__TAURI__.core;

// Store the current documents
let allDocuments = [];
let selectedDocumentId = null;

/**
 * Populate the document list with the provided documents
 */
function populateDocumentList(documents) {
  const container = document.getElementById('document-items-container');
  container.innerHTML = '';
  allDocuments = documents;
  
  // Sort documents based on the selected sort option
  const sortSelect = document.getElementById('sort-select');
  sortDocuments(sortSelect.value);

  // Render each document in the list
  allDocuments.forEach(doc => {
    const itemDiv = document.createElement('div');
    itemDiv.classList.add(
      'document-item', 
      'p-2', 
      'border-b', 
      'border-gray-200', 
      'hover:bg-gray-100', 
      'flex', 
      'justify-between', 
      'items-center',
      'cursor-pointer'
    );
    itemDiv.dataset.docId = doc.id;
    
    // If the document is paused, add visual indication
    if (doc.paused) {
      itemDiv.classList.add('bg-gray-100');
    }
    
    // When clicking on the item, show the document details
    itemDiv.onclick = () => selectDocument(doc.id);

    // Document name
    const nameSpan = document.createElement('span');
    nameSpan.classList.add('flex-grow', 'truncate', 'pr-2');
    nameSpan.textContent = doc.name;
    itemDiv.appendChild(nameSpan);
    
    // Button container
    const btnContainer = document.createElement('div');
    btnContainer.classList.add('flex', 'gap-1');
    
    // Pause/Resume button
    const pauseBtn = document.createElement('button');
    pauseBtn.textContent = doc.paused ? '▶️' : '⏸️';
    pauseBtn.title = doc.paused ? 'Resume' : 'Pause';
    pauseBtn.classList.add(
      'text-xs', 
      'px-2', 
      'py-1', 
      'rounded',
      doc.paused ? 'bg-green-100' : 'bg-yellow-100'
    );
    pauseBtn.onclick = (e) => {
      e.stopPropagation(); // Prevent triggering the item click
      togglePauseDocument(doc.id, !doc.paused);
    };
    
    // Delete button
    const delBtn = document.createElement('button');
    delBtn.textContent = '🗑️';
    delBtn.title = 'Delete';
    delBtn.classList.add(
      'text-xs', 
      'px-2', 
      'py-1', 
      'rounded',
      'bg-red-100'
    );
    delBtn.onclick = (e) => {
      e.stopPropagation(); // Prevent triggering the item click
      if (confirm(`Are you sure you want to delete "${doc.name}"?`)) {
        deleteDocument(doc.id);
      }
    };
    
    btnContainer.appendChild(pauseBtn);
    btnContainer.appendChild(delBtn);
    itemDiv.appendChild(btnContainer);
    
    container.appendChild(itemDiv);
  });
}

/**
 * Sort documents by the specified field
 */
function sortDocuments(sortBy) {
  switch (sortBy) {
    case 'name':
      allDocuments.sort((a, b) => a.name.localeCompare(b.name));
      break;
    case 'date':
      allDocuments.sort((a, b) => new Date(b.created_at) - new Date(a.created_at));
      break;
    case 'model':
      allDocuments.sort((a, b) => a.embedding_model_name.localeCompare(b.embedding_model_name));
      break;
  }
}

/**
 * Select a document and display its details
 */
function selectDocument(docId) {
  // Update the selected state
  selectedDocumentId = docId;
  
  // Clear previous selection visual indicator
  document.querySelectorAll('.document-item').forEach(item => {
    item.classList.remove('bg-blue-100', 'border-l-4', 'border-blue-500');
  });
  
  // Add visual indicator to selected item
  const selectedItem = document.querySelector(`.document-item[data-doc-id="${docId}"]`);
  if (selectedItem) {
    selectedItem.classList.add('bg-blue-100', 'border-l-4', 'border-blue-500');
  }
  
  // Find the document
  const doc = allDocuments.find(d => d.id === docId);
  if (!doc) return;
  
  // Update the detail view
  document.getElementById('no-selection').classList.add('hidden');
  document.getElementById('document-details').classList.remove('hidden');
  
  document.getElementById('detail-name').value = doc.name;
  document.getElementById('detail-date').textContent = formatDate(doc.created_at);
  document.getElementById('detail-model').textContent = doc.embedding_model_name;
  document.getElementById('detail-notes').value = doc.notes || '';
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
  
  try {
    // Call backend to update document (note: you'll need to implement this Rust function)
    await invoke('update_document_details', { 
      docId: selectedDocumentId.toString(),
      name: newName,
      notes: newNotes
    });
    
    // Update local data
    const docIndex = allDocuments.findIndex(d => d.id === selectedDocumentId);
    if (docIndex >= 0) {
      allDocuments[docIndex].name = newName;
      allDocuments[docIndex].notes = newNotes;
      
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
    sortDocuments(e.target.value);
    populateDocumentList(allDocuments);
    
    // Reselect the current document if there was one
    if (selectedDocumentId) {
      selectDocument(selectedDocumentId);
    }
  });
  
  // Set up event listener for save button
  document.getElementById('detail-save-btn').addEventListener('click', saveDocumentChanges);
  
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
