import { Mark, mergeAttributes } from '@tiptap/core'
import { Plugin } from '@tiptap/pm/state'

export const BasicHighlightMark = Mark.create({
  name: 'basicHighlight',
  
  // Mark specific properties
  inclusive: true,
  excludes: '',
  
  // Attributes
  addAttributes() {
    return {
      id: {
        default: null
      },
      color: {
        default: 'blue'
      }
    }
  },
  
  // Parse from HTML
  parseHTML() {
    return [
      {
        tag: 'mark[data-type="basic-highlight"]'
      }
    ]
  },
  
  // Render to HTML
  renderHTML({ HTMLAttributes }) {
    return [
      'mark', 
      mergeAttributes(
        { 'data-type': 'basic-highlight' },
        { class: `basic-highlight basic-highlight-${HTMLAttributes.color}` },
        HTMLAttributes
      ), 
      0
    ]
  },
  
  // Add commands - including the ability to insert pre-highlighted text
  addCommands() {
    return {
      setBasicHighlight: attributes => ({ chain }) => {
        return chain()
          .setMark(this.name, attributes)
          .run()
      },
      
      toggleBasicHighlight: attributes => ({ chain }) => {
        return chain()
          .toggleMark(this.name, attributes, { extendEmptyMarkRange: true })
          .run()
      },
      
      unsetBasicHighlight: () => ({ chain }) => {
        return chain()
          .unsetMark(this.name, { extendEmptyMarkRange: true })
          .run()
      },
      
      // Command to insert text with the highlight already applied
      insertHighlightedText: (text, attributes = {}) => ({ chain, state, schema }) => {
        // Generate ID if not provided
        const id = attributes.id || `highlight-${Date.now()}`
        const color = attributes.color || 'blue'
        
        // Insert text then apply highlight
        return chain()
          .insertContent(text)
          .setTextSelection({
            from: state.selection.from - text.length,
            to: state.selection.from
          })
          .setMark(this.name, { id, color })
          .run()
      }
    }
  },
  
  // Add keyboard shortcuts
  addKeyboardShortcuts() {
    return {
      'Mod-h': () => this.editor.commands.toggleBasicHighlight({ color: 'blue' })
    }
  },
  
  // Add event handlers for clicks
  addProseMirrorPlugins() {
    return [
      new Plugin({
        props: {
          handleClick(view, pos, event) {
            // Find highlight marks at the clicked position
            const { doc, schema } = view.state
            const resolvedPos = doc.resolve(pos)
            const marks = resolvedPos.marks()
            
            for (const mark of marks) {
              if (mark.type === schema.marks.basicHighlight) {
                // Emit a custom event for external handling
                const customEvent = new CustomEvent('highlightClicked', {
                  detail: { 
                    id: mark.attrs.id, 
                    color: mark.attrs.color,
                    text: doc.textBetween(
                      Math.max(0, resolvedPos.start()),
                      Math.min(doc.content.size, resolvedPos.end())
                    )
                  }
                })
                document.dispatchEvent(customEvent)
                
                return true
              }
            }
            
            return false
          }
        }
      })
    ]
  }
})

// CSS for the highlighting:
/*
.basic-highlight {
  background-color: transparent;
  position: relative;
  cursor: pointer;
  border-radius: 0;
  padding: 0;
  margin: 0;
}

.basic-highlight::before {
  content: "";
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  left: 0;
  background-color: rgba(0, 123, 255, 0.2);
  border-radius: 0.1em;
  z-index: -1;
}

.basic-highlight-blue::before {
  background-color: rgba(0, 123, 255, 0.2);
}

.basic-highlight-green::before {
  background-color: rgba(46, 204, 64, 0.2);
}

.basic-highlight-yellow::before {
  background-color: rgba(255, 220, 0, 0.3);
}

.basic-highlight-red::before {
  background-color: rgba(255, 65, 54, 0.2);
}
*/

export default BasicHighlightMark