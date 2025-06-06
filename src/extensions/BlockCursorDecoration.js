import { Extension } from '@tiptap/core'
import { Decoration, DecorationSet } from 'prosemirror-view'
import { Plugin, PluginKey } from 'prosemirror-state'

export const BlockCursorDecoration = Extension.create({
  name: 'blockCursorDecoration',

  addProseMirrorPlugins() {
    return [
      new Plugin({
        key: new PluginKey('blockCursorDecoration'),
        props: {
          decorations: state => {
            // Only show when selection is a cursor (not a range)
            const { empty, from } = state.selection
            if (!empty) return null

            return DecorationSet.create(state.doc, [
              Decoration.widget(from, () => {
                const span = document.createElement('span')
                span.className = 'block-cursor'
                return span
              }, { side: -1 }) // side: -1 puts it before the cursor
            ])
          }
        }
      })
    ]
  }
})