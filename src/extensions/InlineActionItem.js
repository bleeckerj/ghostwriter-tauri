import { Node } from '@tiptap/core'
import { Plugin, PluginKey } from 'prosemirror-state'

export const InlineActionItem = Node.create({
  name: 'inlineActionItem',
  
  group: 'inline',
  inline: true,
  selectable: true,
  draggable: true,

  parseHTML() {
    return [
      {
        tag: 'span[data-type="inline-action"]'
      }
    ]
  },

  renderHTML({ HTMLAttributes }) {
    return ['span', { 
      ...HTMLAttributes,
      'data-type': 'inline-action',
      class: 'inline-action-item'
    }, 'INCANT']
  },

  addProseMirrorPlugins() {
    let timeout
    let nflTag = null
    const showTag = (view, coords) => {
        // Remove any existing tag first
        if (nflTag) {
          nflTag.remove()
        }
  
        nflTag = document.createElement('span')
        nflTag.textContent = 'INCANT'
        nflTag.className = 'inline-action-item'
        nflTag.style.position = 'absolute'
        nflTag.style.left = `${coords.right + 5}px`
        nflTag.style.top = `${coords.top}px`
        nflTag.style.opacity = '0'  // Start invisible
        
        // Add click handler
        nflTag.addEventListener('click', () => {
          const tr = view.state.tr
          const { selection } = view.state
          tr.insertText(' Sample text ', selection.from)
          view.dispatch(tr)
        })
        
        document.body.appendChild(nflTag)
        
        // Trigger fade in
        requestAnimationFrame(() => {
          nflTag.style.opacity = '1'
        })
      }
  
      const hideTag = () => {
        if (nflTag) {
          nflTag.style.opacity = '0'
          // Remove after fade out animation completes
          setTimeout(() => nflTag?.remove(), 200)
          nflTag = null
        }
      }
    return [
      new Plugin({
        key: new PluginKey('inlineActionItem'),
        
        view(editorView) {
          return {
            update: (view, prevState) => {
              const { selection } = view.state
            
            // Clear existing timeout
            if (timeout) {
                clearTimeout(timeout)
            }

            // Hide existing tag when selection changes
            hideTag()

            timeout = setTimeout(() => {
                const coords = view.coordsAtPos(selection.from)
                showTag(view, coords)
              }, 5000)  // Adjust delay as needed (currently 1 second)
            },
            destroy: () => {
              // Cleanup on editor destroy
              if (timeout) {
                clearTimeout(timeout)
              }
              hideTag()
            }
          }
        }
      })
    ]
  }
})