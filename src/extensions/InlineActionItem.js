import { Node } from '@tiptap/core'
import { Plugin, PluginKey } from 'prosemirror-state'

export const InlineActionItem = Node.create({
  name: 'inlineActionItem',

  addOptions() {
    return {
      disabled: false,
      timeout: 10000,
      onClick: () => null,
    }
  },
  
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
    }, ' ']
  },

  addProseMirrorPlugins() {
    const options = this.options
    let waitingForTyping = false

    return [
      new Plugin({
        key: new PluginKey('inlineActionItem'),
        
        props: {
          handleClick(view, pos, event) {
            const node = view.state.doc.nodeAt(pos)
            if (node && node.type.name === 'inlineActionItem') {
              if (typeof options.onClick === 'function') {
                const tr = view.state.tr.delete(pos, pos + node.nodeSize)
                view.dispatch(tr)
                waitingForTyping = true
                
                options.onClick(view, pos, event)
              }
              return true
            }
            return false
          },
        },

        view(editorView) {
          let timeout
          return {
            update: (view, prevState) => {
              if (timeout) clearTimeout(timeout)
              
              // Reset waitingForTyping if content has changed
              if (prevState && !view.state.doc.eq(prevState.doc)) {
                waitingForTyping = false
              }
              
              // Check the options.disabled flag directly
              if (options.disabled || waitingForTyping) return
              
              const { selection } = view.state
              if (!view.state.doc.textContent.trim().length) return

              if (prevState && selection.eq(prevState.selection)) return

              // Check for existing button
              let buttonExists = false
              view.state.doc.descendants((node) => {
                if (node.type.name === 'inlineActionItem') {
                  buttonExists = true
                  return false
                }
              })
              
              if (!buttonExists) {
                timeout = setTimeout(() => {
                  const node = view.state.schema.nodes.inlineActionItem.create()
                  const tr = view.state.tr.insert(selection.from, node)
                  view.dispatch(tr)
                }, options.timeout)
              }
            },
            destroy: () => {
              if (timeout) clearTimeout(timeout)
            }
          }
        }
      })
    ]
  }
})