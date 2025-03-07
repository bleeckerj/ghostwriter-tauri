// RichLogEntryNode.js
import { Node } from '@tiptap/core'
import { mergeAttributes } from '@tiptap/core'

const RichLogEntryNode = Node.create({
  name: 'richLogEntry',
  
  group: 'block',
  
  content: '', // Makes the node a leaf node (no nested content)
  
  selectable: true, // Allow selection for copy/paste
  draggable: false, // Prevent dragging
  
  atom: true, // Treat the node as a single unit
  
  addOptions() {
    return {
      onDelete: () => {
        // Default delete behavior (do nothing)
        console.warn('No onDelete handler provided for RichLogEntryNode')
      },
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
      }
    }
  },

  parseHTML() {
    return [
      {
        tag: 'div[data-type="rich-log-entry"]',
      },
    ]
  },

  renderHTML({ HTMLAttributes }) {
    return ['div', mergeAttributes(
      { 'data-type': 'rich-log-entry', class: 'rich-log-entry' },
      HTMLAttributes
    )]
  },

  addNodeView() {
    return ({ node, getPos, editor }) => {
      const dom = document.createElement('div')
      dom.classList.add('rich-log-entry', `rich-log-level-${node.attrs.level}`)
      
      // Timestamp
      const timestamp = document.createElement('span')
      timestamp.classList.add('rich-log-timestamp')
      timestamp.textContent = node.attrs.timestamp // Use the timestamp string directly
 
      // Message
      const message = document.createElement('span')
      message.classList.add('rich-log-message')
      message.textContent = node.attrs.message
      
      // Delete button
      const deleteBtn = document.createElement('button')
      deleteBtn.classList.add('rich-log-delete-btn')
      deleteBtn.textContent = 'DEL'
      
      // Use the onDelete option from the node's options
      deleteBtn.onclick = (e) => {
        e.preventDefault()
        this.options.onDelete({ node, getPos, editor }) // Pass node, getPos, and editor
      }

      const pauseRagBtn = document.createElement('button')
      pauseRagBtn.classList.add('diagnostics-area-button')
      pauseRagBtn.classList.add('enabled')
      pauseRagBtn.textContent = 'PAUSE'
      
      // Append elements
      dom.appendChild(timestamp)
      dom.appendChild(message)
      dom.appendChild(deleteBtn)
      dom.appendChild(pauseRagBtn)
      
      return {
        dom,
        destroy: () => {
          // Clean up any event listeners if needed
        },
      }
    }
  },
})

export default RichLogEntryNode