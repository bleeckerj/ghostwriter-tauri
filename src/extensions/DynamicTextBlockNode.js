import { Node, mergeAttributes } from '@tiptap/core'

const DynamicTextBlockNode = Node.create({
  name: 'dynamicTextBlockNode', // New Node name

  group: 'inline', // This makes it a block-level node
  inline: true,     // ✅ Ensures it doesn’t break onto a new line
  content: 'text*', // Allows inline text inside
  selectable: true, // Makes it selectable
  draggable: false, // Can set to true if you want drag functionality

  addAttributes() {
    return {
    //   textColor: {
    //     default: 'white',
    //     parseHTML: element => element.getAttribute('data-color'),
    //     renderHTML: attributes => ({
    //       'data-color': attributes.textColor,
    //       style: `color: ${attributes.textColor};`,
    //     })
    //   },
    //   backgroundColor: {
    //     default: 'transparent',
    //     parseHTML: element => element.getAttribute('data-bg-color'),
    //     renderHTML: attributes => ({
    //       'data-bg-color': attributes.backgroundColor,
    //       style: `background-color: ${attributes.backgroundColor};`,
    //     })
    //   },
      twMisc: {
        default: 'animated-highlight bg-amber-200 hover:bg-red-500',
        parseHTML: element => element.getAttribute('data-tw-misc'),
        renderHTML: attributes => ({
          'data-tw-misc': attributes.twMisc,
          class: "dynamic-text-block ${attributes.twMisc}",  // This will apply the Tailwind classes    
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
        tag: 'span[data-type="dynamic-text-block"]'
      }
    ]
  },

  renderHTML({ HTMLAttributes }) {
    return [
      'span',
      mergeAttributes(
        { 'data-type': 'dynamic-text-block' },
        {
          class: [
            'dynamic-text-block',  // ✅ Always include base class
            ...(HTMLAttributes['data-tw-misc'] ? HTMLAttributes['data-tw-misc'].split(' ') : []),  // ✅ Properly extract `twMisc`
          ].join(' '),  // ✅ Convert array into a valid class string
        },
        HTMLAttributes
      ),
      0,
    ];
  },

   // ✅ This enables click detection while keeping text editable
//    addKeyboardShortcuts() {
//     return {
//       'Mod-Shift-H': () => this.editor.commands.toggleNode(this.name, 'paragraph'),
//     };
//   },

  

//   addNodeView() {
//     return ({ node, HTMLAttributes, getPos, editor }) => {
//       const dom = document.createElement('span');
//       dom.classList.add('dynamic-text-block');
//       if(node.attrs.twMisc) {
//         node.attrs.twMisc.split(' ').forEach(cls => dom.classList.add(cls));
//       }

//       dom.setAttribute('data-type', 'dynamic-text-block');
//       dom.setAttribute('contenteditable', 'true'); // ✅ Ensure ProseMirror makes it editable

//       // ✅ APPLY ALL ATTRIBUTES TO THE ELEMENT
//       Object.entries(node.attrs).forEach(([key, value]) => {
//         if (value !== null) {
//           dom.setAttribute(`data-${key}`, value);
//         }
//       });

//       dom.style.color = node.attrs.textColor;
//       dom.style.backgroundColor = node.attrs.backgroundColor;

//       dom.innerText = node.textContent;


//     //   // ✅ Click Event Handler
//     //   dom.addEventListener('click', (event) => {
//     //     event.stopPropogation();
//     //     alert(`Clicked on: ${node.textContent}`);
//     //   });

//       // ✅ Ensure edits update ProseMirror state
//       dom.addEventListener('input', () => {
//         const transaction = editor.state.tr.setNodeMarkup(
//           getPos(),
//           null,
//           node.attrs,
//           editor.schema.text(dom.innerText)
//         );
//         editor.view.dispatch(transaction);
//       });


//       return { dom };
//     };
//   },
});

export default DynamicTextBlockNode;
