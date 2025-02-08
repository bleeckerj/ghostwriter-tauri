// DiagnosticLogEntryNode.js
import { Node } from '@tiptap/core'
import { mergeAttributes } from '@tiptap/core'

export const DiagnosticLogEntryNode = Node.create({
  name: 'logEntry',
  
  group: 'block',
  
  content: '', // Makes the node a leaf node (no nested content)
  
  selectable: true, // Allow selection for copy/paste
  draggable: false, // Prevent dragging
  
  atom: true, // Treat the node as a single unit
  
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
      level: {
        default: 'info',
      }
    }
  },

  parseHTML() {
    return [
      {
        tag: 'div[data-type="log-entry"]',
      },
    ]
  },

  renderHTML({ HTMLAttributes }) {
    return ['div', mergeAttributes(
      { 'data-type': 'log-entry', class: 'log-entry' },
      HTMLAttributes
    )]
  },

  addNodeView() {
    return ({ node, getPos }) => {
      const dom = document.createElement('div')
      dom.classList.add('log-entry', `log-level-${node.attrs.level}`)
      
      // Timestamp
      const timestamp = document.createElement('span')
      timestamp.classList.add('log-timestamp')
      timestamp.textContent = new Date(node.attrs.timestamp).toLocaleString()
      
      // Message
      const message = document.createElement('span')
      message.classList.add('log-message')
      message.textContent = node.attrs.message
      
      // Delete button
      const deleteBtn = document.createElement('button')
      deleteBtn.classList.add('log-delete-btn')
      deleteBtn.textContent = 'DEL'
      deleteBtn.onclick = (e) => {
        e.preventDefault()
        // Dispatch a custom event that we'll handle in the editor setup
        const event = new CustomEvent('log-entry-delete', {
          detail: { id: node.attrs.id, pos: getPos() }
        })
        dom.dispatchEvent(event)
      }
      
      // Append elements
      dom.appendChild(timestamp)
      dom.appendChild(message)
      dom.appendChild(deleteBtn)
      
      return {
        dom,
        destroy: () => {
          // Clean up any event listeners if needed
        },
      }
    }
  },
})

