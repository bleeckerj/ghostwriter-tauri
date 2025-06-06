import { Mark, mergeAttributes } from '@tiptap/core'

export default Mark.create({
  name: 'ghostCompletionMark',
  addOptions() {
    return {
      HTMLAttributes: {
        class: 'ghost-completion',
        // style: 'color: #aaa; background-color: #eee; font-style: italic;'
      },
    }
  },
  parseHTML() {
    return [
      {
        tag: 'span.ghost-completion',
      },
    ]
  },
  renderHTML({ HTMLAttributes }) {
    return ['span', mergeAttributes(this.options.HTMLAttributes, HTMLAttributes), 0]
  },
})