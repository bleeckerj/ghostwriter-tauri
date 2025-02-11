import { Node } from '@tiptap/core'
import { Plugin, PluginKey } from 'prosemirror-state'

export const InlineActionItem = Node.create({
  name: 'inlineActionItem',
  
  group: 'inline',
  inline: true,
  selectable: false,
  draggable: false,
  atom: true,

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
    }, 'NFL']
  },

  addProseMirrorPlugins() {
    let timeout
    let waitingForTyping = false

    return [
      new Plugin({
        key: new PluginKey('inlineActionItem'),
        
        props: {
          handleClick(view, pos, event) {
            const node = view.state.doc.nodeAt(pos)
            if (node && node.type.name === 'inlineActionItem') {
              const tr = view.state.tr
              tr.insertText(' Sample text ', pos + node.nodeSize)
              tr.delete(pos, pos + node.nodeSize)
              view.dispatch(tr)
              waitingForTyping = true
              return true
            }
            return false
          },

          handleKeyDown(view, event) {
            if (waitingForTyping && (event.key.length === 1 || event.key === 'Enter' || event.key === 'Backspace')) {
              waitingForTyping = false
            }
            return false
          }
        },

        view(editorView) {
          return {
            update: (view, prevState) => {
              if (timeout) {
                clearTimeout(timeout)
              }

              // Check if we just deleted an NFL tag by comparing the old and new state
              if (prevState) {
                let hadTag = false
                let hasTag = false
                
                // Check old state
                prevState.doc.descendants((node, pos) => {
                  if (node.type.name === 'inlineActionItem') {
                    hadTag = true
                    return false
                  }
                })
                
                // Check new state
                view.state.doc.descendants((node, pos) => {
                  if (node.type.name === 'inlineActionItem') {
                    hasTag = true
                    return false
                  }
                })

                // If we had a tag but don't anymore, reset waiting state
                if (hadTag && !hasTag) {
                  waitingForTyping = false
                }
              }

              if (waitingForTyping) {
                return
              }

              // Only proceed if no tag exists
              let tagExists = false
              view.state.doc.descendants((node, pos) => {
                if (node.type.name === 'inlineActionItem') {
                  tagExists = true
                  return false
                }
              })

              if (!tagExists) {
                timeout = setTimeout(() => {
                  const { selection } = view.state
                  const node = view.state.schema.nodes.inlineActionItem.create()
                  const tr = view.state.tr.insert(selection.from, node)
                  view.dispatch(tr)
                }, 1000)
              }
            },
            destroy: () => {
              if (timeout) {
                clearTimeout(timeout)
              }
            }
          }
        }
      })
    ]
  }
})