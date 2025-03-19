import { Node } from '@tiptap/core'
import { Plugin, PluginKey } from 'prosemirror-state'

export const ClickableNode = Node.create({
  name: 'clickableNode',
  
  group: 'inline',
  
  inline: true,
  
  atom: false,
  
  addAttributes() {
    return {
      id: {
        default: '',
      },
    }
  },
  
  parseHTML() {
    return [
      {
        tag: 'span.clickable-node',
      },
    ]
  },
  
  renderHTML({ HTMLAttributes }) {
    return ['span', { class: 'clickable-node', ...HTMLAttributes }]
  },
  
  addNodeView() {
    return ({ node, getPos, editor }) => {
      // Create container element
      const dom = document.createElement('div')
      dom.className = 'editable-clickable-node'
      
      // Create content element that will be editable
      const contentDOM = document.createElement('div')
      contentDOM.className = 'editable-content'
      dom.appendChild(contentDOM)
      
      // Add click handler to the container
      dom.addEventListener('click', (event) => {
        // Only handle clicks on the container, not the content
        if (event.target === dom) {
          event.preventDefault()
          console.log('Node clicked!', node.attrs.id)
          editor.emit('editableClickableNode:click', { id: node.attrs.id })
        }
      })
      
      return {
        dom,
        contentDOM, // This is crucial for making the content editable
        
        update(updatedNode) {
          if (updatedNode.type.name !== 'editableClickableNode') return false
          // Update if needed
          return true
        },
        
        destroy() {
          // Clean up event listeners
        }
      }
    }
  },
  
  // Optionally add commands to insert/manage your node
  addCommands() {
    return {
      insertClickableNode: (attributes = {}) => ({ chain }) => {
        return chain()
          .insertContent({
            type: this.name,
            attrs: attributes,
          })
          .run()
      },
    }
  },
})