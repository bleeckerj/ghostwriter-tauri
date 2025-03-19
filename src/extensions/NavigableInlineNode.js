import { Node } from '@tiptap/core'

export const NavigableInlineNode = Node.create({
  name: 'navigableInlineNode',
  
  // Make it inline
  inline: true,
  group: 'inline',
  
  // Only allow text content
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
    return [{ tag: 'span.navigable-inline-node' }]
  },
  
  // Render to HTML - add zero-width spaces around the node
  renderHTML({ HTMLAttributes }) {
    return ['span', { class: 'navigable-inline-node', ...HTMLAttributes }, 0]
  },
  
  // Node view
  addNodeView() {
    return ({ node, getPos, editor }) => {
      // Create the main container
      const dom = document.createElement('span')
      dom.className = 'navigable-inline-node'
      
      // Create the editable content area
      const contentDOM = document.createElement('span')
      contentDOM.className = 'node-content'
      
      // Create a button
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
        
        // Visual feedback
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
  
  // Add commands
  addCommands() {
    return {
      addNavigableNode: (attrs = {}, text = 'Click me') => ({ chain, commands }) => {
        // Add a space after
        return chain()
          // Insert the node
          .insertContent({
            type: this.name,
            attrs,
            content: [{ type: 'text', text }]
          })
          // Move cursor after the node and insert a space if at end of line
          .command(({ tr, dispatch }) => {
            if (dispatch) {
              const { selection } = tr
              // Add a space after if we're at the end of a line
              const needsSpace = tr.doc.textBetween(
                selection.from, 
                Math.min(selection.from + 1, tr.doc.content.size),
                ' '
              ).trim().length === 0
              
              if (needsSpace) {
                tr.insertText(' ', selection.from)
              }
            }
            return true
          })
          .run()
      }
    }
  }
})

// Add this CSS to your stylesheet:
/*
.navigable-inline-node {
  display: inline-flex;
  align-items: center;
  background-color: #e6f2ff;
  border: 1px solid #0074d9;
  border-radius: 3px;
  padding: 0 4px;
  margin: 0 1px;
  position: relative;
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

.ProseMirror .navigable-inline-node {
  padding-left: 1px;
  padding-right: 1px;
}

.ProseMirror .navigable-inline-node::before,
.ProseMirror .navigable-inline-node::after {
  content: "";
  display: inline-block;
  width: 1px;
  color: transparent;
  pointer-events: none;
}
*/

export default NavigableInlineNode