import { Extension } from '@tiptap/core'
import { Decoration, DecorationSet } from 'prosemirror-view'
import { Plugin, PluginKey } from 'prosemirror-state'

export const GhostCompletionDecoration = Extension.create({
  name: 'ghostCompletionDecoration',

  addOptions() {
    return {
      suggestion: '',
    }
  },

  addProseMirrorPlugins() {
    return [
      new Plugin({
        key: new PluginKey('ghostCompletionDecoration'),
        props: {
          decorations: state => {
            const suggestion = this.options.suggestion
            if (!suggestion) return null
            const { from } = state.selection
            return DecorationSet.create(state.doc, [
              Decoration.widget(from, () => {
                const span = document.createElement('span')
                span.className = 'ghost-completion'
                span.textContent = suggestion
                // span.style.opacity = '0.5'
                // span.style.pointerEvents = 'none'
                // span.style.color = '#aaa'
                return span
              }),
            ])
          },
        },
      }),
    ]
  },
})