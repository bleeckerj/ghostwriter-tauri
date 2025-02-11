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
    return [
      new Plugin({
        key: new PluginKey('inlineActionItem'),
        
        view(editorView) {
          return {
            update: (view, prevState) => {
              const { selection } = view.state
              
              // Only update if selection changed
              if (prevState && selection.eq(prevState.selection)) {
                return
              }
              
              // Remove existing NFL tags
              const existingTags = document.querySelectorAll('.inline-action-item')
              existingTags.forEach(tag => tag.remove())
              
              // Create new NFL tag at cursor
              const coords = view.coordsAtPos(selection.from)
              const nflTag = document.createElement('span')
              nflTag.textContent = 'INCANT'
              nflTag.className = 'inline-action-item'
              nflTag.style.position = 'absolute'
              nflTag.style.left = `${coords.right + 5}px`  // Small offset from cursor
              nflTag.style.top = `${coords.top}px`
              
              // Add click handler
              nflTag.addEventListener('click', () => {
                const tr = view.state.tr
                const insertPos = selection.from
                tr.insertText(' Sample text ', insertPos)
                view.dispatch(tr)
              })
              
              document.body.appendChild(nflTag)
            }
          }
        }
      })
    ]
  }
})