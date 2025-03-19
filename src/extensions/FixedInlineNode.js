import { Node } from '@tiptap/core'

export const FixedInlineNode = Node.create({
  name: 'fixedInlineNode',
  
  // Make it inline
  inline: true,
  group: 'inline',
  
  // Only allow text content - no paragraphs!
  content: 'text*',
  
  // Attributes
  addAttributes() {
    return {
      id: {
        default: ''
      }
    }
  },
  
  // Parse from HTML
  parseHTML() {
    return [{ tag: 'span.fixed-inline-node' }]
  },
  
  // Render to HTML
  renderHTML({ HTMLAttributes }) {
    return ['span', { class: 'fixed-inline-node', ...HTMLAttributes }, 0]
  },
  
  // Node view
  addNodeView() {
    return ({ node, getPos, editor }) => {
      // Create the main container
      const dom = document.createElement('span')
      dom.className = 'fixed-inline-node'
      
      // Create the editable content area
      const contentDOM = document.createElement('span')
      contentDOM.className = 'node-content'
      
      // Create a simple button element
      const button = document.createElement('button')
      button.type = 'button'
      button.className = 'node-button'
      button.textContent = '•'
      button.setAttribute('contenteditable', 'false')
      
      // Add elements to the DOM
      dom.appendChild(contentDOM)
      dom.appendChild(button)
      
      // Add the click handler
      button.addEventListener('click', (e) => {
        e.preventDefault()
        e.stopPropagation()
        console.log('Button clicked! Node ID:', node.attrs.id)
        
        button.style.backgroundColor = 'red'
        setTimeout(() => button.style.backgroundColor = '', 300)
        
        // Emit an event
        editor.emit('node:clicked', {
          id: node.attrs.id,
          node,
          getPos
        })
      })
      
      return {
        dom,
        contentDOM
      }
    }
  },
  
  // Add command to insert the node - FIXED CONTENT STRUCTURE
  addCommands() {
    return {
      addFixedNode: (attrs = {}, text = 'Click me') => ({ chain }) => {
        return chain().insertContent({
          type: this.name,
          attrs,
          content: [
            {
              type: 'text',
              text: text
            }
          ]
        }).run()
      }
    }
  }
})

// Add this CSS to your stylesheet:
/*
.fixed-inline-node {
  display: inline-flex;
  align-items: center;
  background-color: #e6f2ff;
  border: 1px solid #0074d9;
  border-radius: 3px;
  padding: 0 4px;
  margin: 0 2px;
}

.node-content {
  min-width: 1em;
}

.node-button {
  width: 16px;
  height: 16px;
  padding: 0;
  margin-left: 4px;
  background-color: #0074d9;
  color: white;
  border: none;
  border-radius: 50%;
  font-size: 12px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: all;
}
*/

export default FixedInlineNode