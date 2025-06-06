import { Extension } from '@tiptap/core'
import { Decoration, DecorationSet } from 'prosemirror-view'
import { Plugin, PluginKey } from 'prosemirror-state'

export const BlockOverCursorDecoration = Extension.create({
  name: 'blockOverCursorDecoration',

  addProseMirrorPlugins() {
    return [
      new Plugin({
        key: new PluginKey('blockOverCursorDecoration'),
        props: {
          decorations: state => {
            const { empty, from } = state.selection
            if (!empty || from === 0) return null

            const charPos = from - 1
            const char = state.doc.textBetween(charPos, from, '', '')

            if (!char) return null

            // Inline decoration replaces the character with a styled span
            return DecorationSet.create(state.doc, [
              Decoration.inline(charPos, from, { class: 'block-over-cursor' })
            ])
          }
        }
      })
    ]
  }
})