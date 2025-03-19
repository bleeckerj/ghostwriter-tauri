import { Node } from '@tiptap/core'

export const TextContourNode = Node.create({
  name: 'textContourNode',
  
  // Make it inline
  inline: true,
  group: 'inline',
  
  // Allow text content only
  content: 'text*',
  
  // Attributes
  addAttributes() {
    return {
      id: {
        default: ''
      },
      active: {
        default: false,
        parseHTML: element => element.getAttribute('data-active') === 'true',
        renderHTML: attributes => {
          if (!attributes.active) {
            return {}
          }
          return {
            'data-active': 'true'
          }
        }
      }
    }
  },
  
  // Parse from HTML
  parseHTML() {
    return [{ tag: 'span.text-contour-node' }]
  },
  
  // Render to HTML
  renderHTML({ HTMLAttributes }) {
    return ['span', { class: 'text-contour-node', ...HTMLAttributes }, 0]
  },
  
  // Node view
  addNodeView() {
    return ({ node, getPos, editor }) => {
      // Create container that wraps precisely around text
      const dom = document.createElement('span')
      dom.className = 'text-contour-node'
      if (node.attrs.active) {
        dom.classList.add('active')
      }
      
      // Create content area - this will contain the text
      const contentDOM = document.createElement('span')
      contentDOM.className = 'node-content'
      
      // Add content to the DOM
      dom.appendChild(contentDOM)
      
      // Make the node clickable
      const handleClick = (e) => {
        // Only handle clicks directly on this node, not child nodes
        if (e.target === dom || dom.contains(e.target)) {
          e.preventDefault()
          e.stopPropagation()
          
          console.log('Node clicked:', node.attrs.id)
          
          // Emit an event
          editor.emit('textContour:clicked', {
            id: node.attrs.id,
            node,
            getPos
          })
          
          // Toggle active state
          if (typeof getPos === 'function') {
            editor.view.dispatch(
              editor.view.state.tr.setNodeAttribute(
                getPos(), 
                'active', 
                !node.attrs.active
              )
            )
          }
        }
      }
      
      // Add the click handler
      dom.addEventListener('click', handleClick)
      
      return {
        dom,
        contentDOM,
        update(updatedNode) {
          if (updatedNode.type.name !== 'textContourNode') return false
          
          // Update active state if it changed
          if (updatedNode.attrs.active !== node.attrs.active) {
            dom.classList.toggle('active', updatedNode.attrs.active)
          }
          
          return true
        },
        destroy() {
          dom.removeEventListener('click', handleClick)
        }
      }
    }
  },
  
  // Add commands
  addCommands() {
    return {
      insertTextContourNode: (attrs = {}, text = 'Highlighted text') => ({ chain }) => {
        return chain()
          .insertContent({
            type: this.name,
            attrs,
            content: [{ type: 'text', text }]
          })
          .run()
      }
    }
  }
})

// CSS for text-selection-like highlighting:


export default TextContourNode