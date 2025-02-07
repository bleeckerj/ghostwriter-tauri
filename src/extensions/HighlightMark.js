// Using Tiptap Extension API
import { Mark } from '@tiptap/core'

export const HighlightMark = Mark.create({
  name: 'highlight',
  
  // Defining how the mark appears in the HTML
  renderHTML() {
    return ['span', { class: 'p-4 bg-green-700 text-white' }, 0]
  },

  // Parsing rules for converting HTML back to ProseMirror
  parseHTML() {
    return [
      {
        tag: 'span.highlight-mark',
      },
    ]
  },

  // Key part: defining inclusive property
  inclusive: false,
})

export default HighlightMark