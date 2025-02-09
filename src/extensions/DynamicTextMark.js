import { Mark } from '@tiptap/core'

const DynamicTextMark = Mark.create({
  name: 'dynamicTextMark',
  inclusive: false,  // This is the key property that prevents style inheritance
  
  addAttributes() {
    return {
      textColor: {
        default: 'white',
        parseHTML: element => element.getAttribute('data-color'),
        renderHTML: attributes => ({
          'data-color': attributes.textColor,
          style: `color: ${attributes.textColor};`,
        })
      },
      backgroundColor: {
        default: 'transparent',
        parseHTML: element => element.getAttribute('data-bg-color'),
        renderHTML: attributes => ({
          'data-bg-color': attributes.backgroundColor,
          style: `background-color: ${attributes.backgroundColor};`,
        })
      },
      twMisc: {
        default: 'animated-highlight bg-blue-200',
        parseHTML: element => element.getAttribute('data-tw-misc'),
        renderHTML: attributes => ({
          'data-tw-misc': attributes.twMisc,
          class: attributes.twMisc,  // This will apply the Tailwind classes
        })
      },
      id: {
        default: null,
        parseHTML: element => element.getAttribute('data-id'),
        renderHTML: attributes => ({
          'data-id': attributes.id
        })
      },
      timestamp: {
        default: null,
        parseHTML: element => element.getAttribute('data-timestamp'),
        renderHTML: attributes => ({
          'data-timestamp': attributes.timestamp
        })
      },
      raw: {
        default: null,
        parseHTML: element => element.getAttribute('data-raw'),
        renderHTML: attributes => ({
          'data-raw': attributes.raw
        })
      },
    }
  },

  parseHTML() {
    return [
      {
        tag: 'span[data-type="dynamic-text"]'
      }
    ]
  },

  renderHTML({ HTMLAttributes }) {
    return ['span', { ...HTMLAttributes, 'data-type': 'dynamic-text' }, 0]
  }
})

export default DynamicTextMark