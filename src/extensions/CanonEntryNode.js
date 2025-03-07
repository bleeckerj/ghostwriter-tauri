// CanonEntryNode.js
import { Node } from '@tiptap/core'
import { mergeAttributes } from '@tiptap/core'

const CanonEntryNode = Node.create({
  name: 'canonEntry',
  
  group: 'block',
  
  content: '', // Makes the node a leaf node (no nested content)
  
  selectable: true, // Allow selection for copy/paste
  draggable: false, // Prevent dragging
  
  atom: true, // Treat the node as a single unit
  
  addOptions() {
    return {
      onDelete: () => {
        // Default delete handler
        console.warn('No onDelete handler provided for CanonEntryNode')
      },
      onTogglePause: () => {
        // Default toggle pause handler
        console.warn('No onTogglePause handler provided for CanonEntryNode')
      }
    }
  },
  
  addAttributes() {
    return {
      id: {
        default: null,
      },
      timestamp: {
        default: null,
      },
      message: {
        default: '',
      },
      data: {
        default: null,
      },
      level: {
        default: 'info',
      },
      paused: {
        default: false, // Track paused state
        parseHTML: element => element.getAttribute('data-paused') === 'true',
        renderHTML: attributes => {
          if (!attributes.paused) {
            return {}
          }
          return {
            'data-paused': 'true'
          }
        }
      }
    }
  },

  parseHTML() {
    return [
      {
        tag: 'div[data-type="canon-entry"]',
      },
    ]
  },

  renderHTML({ HTMLAttributes }) {
    return ['div', mergeAttributes(
      { 'data-type': 'canon-entry', class: 'canon-entry' },
      HTMLAttributes
    )]
  },

  addNodeView() {
    return ({ node, getPos, editor }) => {
      const dom = document.createElement('div')
      dom.classList.add('canon-entry', `canon-entry-level-${node.attrs.level}`)
      
      // Timestamp
      const timestamp = document.createElement('span')
      timestamp.classList.add('canon-entry-timestamp')
      timestamp.textContent = node.attrs.timestamp // Use the timestamp string directly
 
      // Message
      const message = document.createElement('span')
      message.classList.add('canon-entry-message')
      message.textContent = node.attrs.message
      
      // Delete button
      const deleteBtn = document.createElement('button')
      deleteBtn.classList.add('canon-entry-button')
      deleteBtn.classList.add('enabled')
      deleteBtn.classList.add('del')
      deleteBtn.textContent = 'DEL'
      
      // Use the onDelete option from the node's options
      deleteBtn.onclick = (e) => {
        e.preventDefault()
        this.options.onDelete({ node, getPos, editor }) // Pass node, getPos, and editor
      }

      // Pause RAG button - with toggle capability
      const pauseRagBtn = document.createElement('button')
      pauseRagBtn.classList.add('canon-entry-button', 'enabled', 'pause')
      console.log('Attributes are ', node.attrs);
      console.log('Initial paused state:', node.attrs.paused);
      // Store state directly on the button element
      pauseRagBtn.dataset.isPaused = node.attrs.paused ? 'true' : 'false'

      // Set initial button appearance based on the current state
      if (pauseRagBtn.dataset.isPaused === 'true') {
        pauseRagBtn.classList.add('button-in')
        pauseRagBtn.textContent = 'RESUME'
      } else {
        pauseRagBtn.classList.add('button-out')
        pauseRagBtn.textContent = 'PAUSE'
      }
      
      // Toggle button state on click
      pauseRagBtn.onclick = (e) => {
        e.preventDefault()
        
        // Debug the current state
        console.log('Button clicked. Current state:', {
          'paused attribute': node.attrs.paused,
          'button data-isPaused': pauseRagBtn.dataset.isPaused,
          'button text': pauseRagBtn.textContent,
          'has button-in': pauseRagBtn.classList.contains('button-in'),
          'has button-out': pauseRagBtn.classList.contains('button-out')
        });
        
        // Toggle the state using the button's dataset
        const currentlyPaused = pauseRagBtn.dataset.isPaused === 'true'
        const newPausedState = !currentlyPaused
        
        //console.log('Toggling from', currentlyPaused, 'to', newPausedState);
        
        // Update the button's dataset with the new state
        pauseRagBtn.dataset.isPaused = newPausedState ? 'true' : 'false'
        
        // Update visual appearance based on new state
        if (newPausedState) {
          // Switching to paused state
          pauseRagBtn.textContent = 'RESUME'
          pauseRagBtn.classList.remove('button-out')
          pauseRagBtn.classList.add('button-in')
        } else {
          // Switching to active state
          pauseRagBtn.textContent = 'PAUSE'
          pauseRagBtn.classList.remove('button-in')
          pauseRagBtn.classList.add('button-out')
        }
        
        // Try to update the node attribute if possible (will only work in editable editors)
        if (editor.isEditable) {
          const pos = getPos()
          if (typeof pos === 'number') {
            editor.commands.updateAttributes('canonEntry', { paused: newPausedState }, { at: pos })
          }
        }
        
        // Call the toggle handler with the new state
        this.options.onTogglePause({
          node,
          getPos,
          editor,
          isPaused: newPausedState
        })
      }
      
      // Append elements
      dom.appendChild(timestamp)
      dom.appendChild(message)
      dom.appendChild(deleteBtn)
      dom.appendChild(pauseRagBtn)
      
      // Update method to handle external state changes
      const update = (updatedNode) => {
        if (updatedNode.attrs.paused !== node.attrs.paused) {
          if (updatedNode.attrs.paused) {
            pauseRagBtn.classList.remove('button-out')
            pauseRagBtn.classList.add('button-in')
            pauseRagBtn.textContent = 'RESUME'
          } else {
            pauseRagBtn.classList.remove('button-in')
            pauseRagBtn.classList.add('button-out')
            pauseRagBtn.textContent = 'PAUSE'
          }
        }
        return true
      }
      
      return {
        dom,
        update,
        destroy: () => {
          // Clean up any event listeners if needed
        },
      }
    }
  },
})

export default CanonEntryNode