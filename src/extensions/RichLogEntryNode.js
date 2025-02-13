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
    return ({ node, getPos }) => {
      const dom = document.createElement('div')
      dom.classList.add('rich-log-entry', `rich-log-level-${node.attrs.level}`)
      
      // Timestamp
      const timestamp = document.createElement('span')
      timestamp.classList.add('rich-log-timestamp')
      timestamp.textContent = new Date(node.attrs.timestamp)
      .toLocaleString('en-US', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
        hour12: false
      })
      // Message
      const message = document.createElement('span')
      message.classList.add('rich-log-message')
      message.textContent = node.attrs.message
      
      // Delete button
      const deleteBtn = document.createElement('button')
      deleteBtn.classList.add('rich-log-delete-btn')
      deleteBtn.textContent = 'DEL'
      deleteBtn.onclick = (e) => {
        e.preventDefault()
        // Dispatch a custom event that we'll handle in the editor setup
        const event = new CustomEvent('rich-log-entry-delete', {
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

export default RichLogEntryNode