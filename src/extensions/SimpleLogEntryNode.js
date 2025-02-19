// SimpleLogEntryNode.js
import { Node } from '@tiptap/core'
import { mergeAttributes } from '@tiptap/core'
import DOMPurify from 'dompurify';
import { TextSelection } from 'prosemirror-state';
import { NodeSelection } from "prosemirror-state";

const SimpleLogEntryNode = Node.create({
  name: 'simpleLogEntry',
  
  group: 'block',
  
  content: '', // Makes the node a leaf node (no nested content)
  
  selectable: true, // Allow selection for copy/paste
  draggable: false, // Prevent dragging
  
  atom: true, // Treat the node as a single unit
  
  addAttributes() {
    return {
      id: {
        default: null,
      },
      timestamp: {
        default: null,
      },
      message: {
        default: '',
      },
      level: {
        default: 'info',
      }
    }
  },

  parseHTML() {
    return [
      {
        tag: 'div[data-type="simple-log-entry"]',
      },
    ]
  },

  renderHTML({ HTMLAttributes }) {
    return ['div', mergeAttributes(
      { 'data-type': 'simple-log-entry', class: 'simple-log-entry' },
      HTMLAttributes
    )]
  },

  addNodeView() {
    return ({ node, getPos, editor }) => {
      const dom = document.createElement('div')
      dom.classList.add('selectable-node');
      dom.classList.add('simple-log-entry', `simple-log-level-${node.attrs.level}`)
      
      // Timestamp
      const timestamp = document.createElement('span')
      timestamp.classList.add('simple-log-timestamp')
      timestamp.textContent = node.attrs.timestamp // Use the timestamp string directly
       // Message
      const message = document.createElement('div')
      message.classList.add('simple-log-message')
      message.innerHTML = ' ' + DOMPurify.sanitize(node.attrs.message)
      
      // Append elements
      dom.appendChild(timestamp)
      dom.appendChild(message)
      
      dom.addEventListener("click", () => {
        if (typeof getPos === "function") {
          const { state, view } = editor;
          console.log('SimpleLogEntryNode clicked', node);

          // Calculate selection range from start to end of the node
          const startPos = getPos();
          const endPos = startPos + node.nodeSize;
  
          // Create and dispatch a text selection over the entire node
          const transaction = state.tr.setSelection(
            TextSelection.create(state.doc, startPos, endPos - 1)
          );
          view.dispatch(transaction);
        }
      });
  

      return {
        dom:dom,
        destroy: () => {
          // Clean up any event listeners if needed
        },
      }
    }
  },
})
export default SimpleLogEntryNode

