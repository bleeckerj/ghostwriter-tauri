const { invoke } = window.__TAURI__.core;

document.addEventListener('DOMContentLoaded', () => {
  // Get the elements
  const urlPanel = document.getElementById('url-panel');
  const urlCloseBtn = document.getElementById('url-close-btn');
  const ingestUrlBtn = document.getElementById('ingest-url-btn');
  const urlSubmitBtn = document.getElementById('url-submit-btn');
  const urlInput = document.getElementById('url-input');

  // Flag to track panel state
  let isPanelVisible = false;

  // Function to toggle the URL panel
  function toggleUrlPanel() {
    if (isPanelVisible) {
      // Hide panel
      urlPanel.classList.remove('url-panel-visible');
      urlPanel.classList.add('url-panel-hidden');
      ingestUrlBtn.classList.remove('button-in');
      isPanelVisible = false;
    } else {
      // Show panel
      urlPanel.classList.remove('url-panel-hidden');
      urlPanel.classList.add('url-panel-visible');
      ingestUrlBtn.classList.add('button-in');
      isPanelVisible = true;
      // Focus the input field for immediate typing
      setTimeout(() => urlInput.focus(), 100);
    }
  }

  // Function to explicitly hide panel (for close button and submit)
  function hideUrlPanel() {
    urlPanel.classList.remove('url-panel-visible');
    urlPanel.classList.add('url-panel-hidden');
    ingestUrlBtn.classList.remove('button-in');
    isPanelVisible = false;
  }

  // Function to handle URL submission
  function submitUrl() {
    const url = urlInput.value.trim();
    if (url) {
      console.log('Processing URL:', url);
      // Here you would add your URL processing logic
      invoke("ingest_from_url", { url }).then((response) => {
        console.log('Response:', response);
        // Show a success message or handle the response
      }).catch((error) => {
        console.error('Error:', error);
        // Show an error message or handle the error
      });
      // Clear the input field for next use
      urlInput.value = '';
      
      // Hide the panel after submission
      //hideUrlPanel();
    }
  }

  // Set up event listeners
  if (ingestUrlBtn) {
    ingestUrlBtn.addEventListener('click', toggleUrlPanel);
  }
  
  if (urlCloseBtn) {
    urlCloseBtn.addEventListener('click', hideUrlPanel);
  }
  
  if (urlSubmitBtn) {
    urlSubmitBtn.addEventListener('click', submitUrl);
  }

  // Also submit when Enter key is pressed in the input field
  if (urlInput) {
    urlInput.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') {
        submitUrl();
      }
      // Close panel on Escape key
      if (e.key === 'Escape') {
        hideUrlPanel();
      }
    });
  }

  // Initially hide the panel - removed conditional check to ensure it always hides on load
  hideUrlPanel();
});