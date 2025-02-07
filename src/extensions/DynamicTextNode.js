import { Editor } from '@tiptap/core'
import StarterKit from '@tiptap/starter-kit'
import { Node } from '@tiptap/core'
import { Plugin, PluginKey } from 'prosemirror-state'

// Define the custom node
const DynamicTextNode = Node.create({
  name: 'dynamicText',
  group: 'inline', // Change group to inline
  inline: true, // Ensure the node is treated as inline
  content: 'text*',
  inclusive: false,

  addAttributes() {
    return {
      textColor: {
        default: 'black',
        parseHTML: element => element.getAttribute('data-color'),
        renderHTML: attributes => ({
          'data-color': attributes.textColor,
          style: `color: ${attributes.textColor};`,
        })
      },
      id: {
        default: null,
        parseHTML: element => element.getAttribute('data-id'),
        renderHTML: attributes => ({
          'data-id': attributes.id
        })
      }
    }
  },

  renderHTML({ HTMLAttributes }) {
    return ['span', HTMLAttributes, 0]
  },

  parseHTML() {
    return [
      {
        tag: 'span[data-type="dynamic-text"]'
      }
    ]
  },

//   addProseMirrorPlugins() {
//     return [
//       new Plugin({
//         key: new PluginKey('dynamicTextPlugin'),
//         view: () => ({
//           update: (view, prevState) => {
//             // Optional: Add update logic here if needed
//           }
//         })
//       })
//     ]
//   }
})

export default DynamicTextNode