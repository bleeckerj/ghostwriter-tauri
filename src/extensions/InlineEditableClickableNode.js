import { Node, mergeAttributes } from '@tiptap/core'

export const InlineEditableClickableNode = Node.create({
  name: 'inlineEditableClickableNode',
  
  // Configure as inline node
  inline: true,
  group: 'inline',
  content: 'text*', // Only allow text content
  
  // Define attributes
  addAttributes() {
    return {
      id: {
        default: '',
      },
      status: {
        default: 'default',
      }
    }
  },
  
  // Define how it gets parsed from HTML
  parseHTML() {
    return [
      {
        tag: 'span[data-type="inline-editable-node"]',
      },
    ]
  },
  
  // Define how it gets rendered to HTML
  renderHTML({ HTMLAttributes }) {
    return ['span', mergeAttributes(
      { 
        'data-type': 'inline-editable-node', 
        class: 'inline-editable-node' 
      },
      HTMLAttributes
    ), 0] // The 0 is a position marker for content
  },
  
  // Custom node view
  addNodeView() {
    return ({ node, getPos, editor }) => {
      // Create wrapper element
      const dom = document.createElement('span')
      dom.className = 'inline-editable-node'
      dom.setAttribute('data-id', node.attrs.id || '')
      dom.setAttribute('data-status', node.attrs.status || 'default')
      
      // Create content container that will be editable
      const contentDOM = document.createElement('span')
      contentDOM.className = 'inline-node-content'
      
      // Create action button - small and to the right of the content
      const actionButton = document.createElement('button')
      actionButton.className = 'inline-node-action-button'
      actionButton.innerHTML = '•'
      actionButton.contentEditable = 'false' // Prevent editing of the button
      actionButton.title = 'Click to interact'
      
      // Assemble the components
      dom.appendChild(contentDOM)
      dom.appendChild(actionButton)
      
      // Define click handler
      const handleButtonClick = (event) => {
        event.preventDefault()
        event.stopPropagation()
        
        console.log('Inline node clicked:', node.attrs.id)
        
        // Emit a custom event
        editor.emit('inlineEditableNode:click', { 
          id: node.attrs.id,
          node,
          getPos,
          event
        })
        
        // Toggle status as visual feedback
        if (typeof getPos === 'function') {
          const newStatus = node.attrs.status === 'active' ? 'default' : 'active'
          editor.view.dispatch(
            editor.view.state.tr.setNodeAttribute(
              getPos(), 
              'status', 
              newStatus
            )
          )
        }
      }
      
      // Add click handler
      actionButton.addEventListener('click', handleButtonClick)
      
      return {
        dom,
        contentDOM,
        
        // Update method
        update(updatedNode) {
          if (updatedNode.type.name !== 'inlineEditableClickableNode') return false
          
          if (updatedNode.attrs.status !== node.attrs.status) {
            dom.setAttribute('data-status', updatedNode.attrs.status)
          }
          
          if (updatedNode.attrs.id !== node.attrs.id) {
            dom.setAttribute('data-id', updatedNode.attrs.id || '')
          }
          
          return true
        },
        
        // Clean up
        destroy() {
          actionButton.removeEventListener('click', handleButtonClick)
        }
      }
    }
  },
  
  // Add commands
  addCommands() {
    return {
      // Insert node with text
      insertInlineEditableNode: (attributes = {}, text = '') => ({ chain }) => {
        return chain()
          .insertContent({
            type: this.name,
            attrs: attributes,
            content: text ? [{ type: 'text', text }] : []
          })
          .run()
      },
      
      // Update node by ID
      updateInlineEditableNodeById: (id, attributes = {}) => ({ tr, state }) => {
        let hasUpdated = false
        
        state.doc.descendants((node, pos) => {
          if (node.type.name === this.name && node.attrs.id === id) {
            Object.keys(attributes).forEach(attr => {
              tr.setNodeAttribute(pos, attr, attributes[attr])
            })
            hasUpdated = true
            return false // Stop traversal
          }
        })
        
        return hasUpdated
      }
    }
  }
})

// CSS to include separately:
/*
.inline-editable-node {
  display: inline-flex;
  align-items: center;
  position: relative;
  border-radius: 4px;
  margin: 0 2px;
  background-color: rgba(0, 116, 217, 0.1);
  border: 1px solid #0074d9;
  padding: 0 4px;
}

.inline-editable-node[data-status="active"] {
  background-color: rgba(46, 204, 64, 0.1);
  border-color: #2ecc40;
}

.inline-node-content {
  min-width: 1em;
  display: inline;
}

.inline-node-action-button {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background-color: #0074d9;
  color: white;
  border: none;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  margin-left: 4px;
  padding: 0;
  cursor: pointer;
}

.inline-editable-node[data-status="active"] .inline-node-action-button {
  background-color: #2ecc40;
}
*/

export default InlineEditableClickableNode