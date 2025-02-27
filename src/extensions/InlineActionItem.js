import { Node } from '@tiptap/core'
import { Plugin, PluginKey } from 'prosemirror-state'

export const InlineActionItem = Node.create({
  name: 'inlineActionItem',


  setOptions() {
    this.options = {
      ...this.options,
      ...options,
    }
    this.editor.view.dispatch(this.editor.view.state.tr.setMeta('update', true))
  },

  addOptions() {
    return {
      disabled: true,
      timeout: 5000,
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
      class: 'inline-action-item bg-gradient-animated text-[1.5em] rounded-full'
    }, '🥑']
  },

  // renderHTML({ HTMLAttributes }) {
  //   return ['span', { 
  //     ...HTMLAttributes,
  //     'data-type': 'inline-action',
  //     class: 'inline-action-item'
  //   }, ['img', {
  //     src: '/src/images/icons/LaboratoryCircle_Guy_01_CircleTeal.webp', // Use the correct path to your icon
  //     alt: 'NFL Icon',
  //     style: 'width: 25px; height: 25px; vertical-align: middle;', // Adjust size as needed
  //   }]
  // ]
  // },

  addProseMirrorPlugins() {
    const options = this.options
    let waitingForTyping = false // not so sure about this..
    let userTyping = false

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
                //console.log("waitingForTyping set to true on click", waitingForTyping)
                options.onClick(view, pos, event)
              }
              return true
            }
            return false
          },
          handleDOMEvents: {
            keydown: (view, event) => {
              userTyping = true
              return false
            }
          }
        },

        view(editorView) {
          let timeout
          return {
            update: (view, prevState) => {
              if (timeout) clearTimeout(timeout)

              // Reset waitingForTyping if content has changed and userTyping is true
              if (prevState && !view.state.doc.eq(prevState.doc) && userTyping) {
                waitingForTyping = false
                userTyping = false
                console.log("waitingForTyping reset to false on user typing", waitingForTyping)
              }
              
              // Check the options.disabled flag directly
              if (options.disabled || waitingForTyping) return false;
              
              const { selection } = view.state
              if (!view.state.doc.textContent.trim().length) return false;

              if (prevState && selection.eq(prevState.selection)) return false;

              // Check for existing button
              let buttonExists = false
              view.state.doc.descendants((node) => {
                if (node.type.name === 'inlineActionItem') {
                  buttonExists = true
                  return false
                }
              })
              
              if (!buttonExists && options.disabled === false) {
                timeout = setTimeout(() => {
                  // Check the options.disabled flag again because it might have changed
                  if (options.disabled === true) return false;
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