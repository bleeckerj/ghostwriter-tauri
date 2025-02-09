// ProgressNode.js

import { Node } from '@tiptap/core'

// ==========================================
// 1. NODE VIEW CLASS
// This handles the actual DOM rendering and updates
// ==========================================
class ProgressView {
    constructor(node, view, getPos) {
        console.log('ProgressView constructor details:', {
            node: node,
            nodeType: node?.type,
            attrs: node?.attrs,
            marks: node?.marks,
            content: node?.content,
            HTMLAttributes: node?.HTMLAttributes
        })
        this.node = node
        this.view = view
        this.getPos = getPos
        
        // Debug logging
        console.log('ProgressView constructor:', {
          node,
          HTMLAttributes: node?.HTMLAttributes,
          HTMLAttributes: node?.HTMLAttributes
        })
        
        // Initialize default attributes if they're undefined
        this.HTMLAttributes = node?.HTMLAttributes || {
          progress_id: 'default-id',
          current_step: 0,
          total_steps: 20,
          current_file: ''
        }
        
        this.dom = this.createDOM()
        this.contentDOM = null
    
        this.startTimeUpdate()

        // Add start time tracking
        this.startTime = null  // Initialize but don't set time yet
        this.elapsedInterval = null
      }

  createDOM() {
    const container = document.createElement('div')
    container.classList.add('progress-indicator')
    container.setAttribute('data-progress-id', this.node.HTMLAttributes.progress_id)

    // Stats container (progress count + time)
    const statsDiv = document.createElement('div')
    statsDiv.classList.add('progress-stats')

    // Progress count
    const countSpan = document.createElement('span')
    countSpan.classList.add('progress-count')
    countSpan.textContent = `${this.node.HTMLAttributes.current_step}/${this.node.HTMLAttributes.total_steps}`

    // Time
    const timeSpan = document.createElement('span')
    timeSpan.classList.add('progress-time')
    timeSpan.textContent = new Date().toLocaleTimeString()

    statsDiv.appendChild(countSpan)
    statsDiv.appendChild(timeSpan)

    // Add elapsed time span
    const elapsedSpan = document.createElement('span')
    elapsedSpan.classList.add('progress-elapsed')
    elapsedSpan.textContent = '00:00'
    statsDiv.appendChild(elapsedSpan)

    // After adding elapsed span
    const metaSpan = document.createElement('span')
    metaSpan.classList.add('progress-meta')
    metaSpan.textContent = this.node.HTMLAttributes.meta || ''
    statsDiv.appendChild(metaSpan)

    // File name
    const fileDiv = document.createElement('div')
    fileDiv.classList.add('progress-file')
    fileDiv.textContent = this.node.HTMLAttributes.current_file

    // Progress bar
    const progressBarContainer = document.createElement('div')
    progressBarContainer.classList.add('progress-bar-container')
    progressBarContainer.style.width = '100%'
    progressBarContainer.style.height = '4px'
    progressBarContainer.style.backgroundColor = '#eee'

    const progressBar = document.createElement('div')
    progressBar.classList.add('progress-bar')
    // Add transition and gradient
    progressBar.style.cssText = `
        width: ${(this.node.HTMLAttributes.current_step / this.HTMLAttributes.total_steps) * 100}%;
        height: 100%;
        background: linear-gradient(90deg, #007bff, #00ff88);
        transition: width 0.3s ease;
        border-radius: 2px;
        box-shadow: 0 1px 3px rgba(0,0,0,0.2);
    `

    progressBarContainer.appendChild(progressBar)

    // Put it all together
    container.appendChild(statsDiv)
    container.appendChild(fileDiv)
    container.appendChild(progressBarContainer)

    return container
  }

  startTimeUpdate() {
    this.timeInterval = setInterval(() => {
      const timeSpan = this.dom.querySelector('.progress-time')
      if (timeSpan) {
        timeSpan.textContent = new Date().toLocaleTimeString()
      }
    }, 1000)
  }

  startElapsedTimer() {
    this.elapsedInterval = setInterval(() => {
        const elapsedSpan = this.dom.querySelector('.progress-elapsed')
        if (elapsedSpan) {
            const now = new Date()
            const elapsed = Math.floor((now - this.startTime) / 1000) // seconds
            const minutes = Math.floor(elapsed / 60)
            const seconds = elapsed % 60
            elapsedSpan.textContent = `Elapsed: ${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`
        }
    }, 1000)
  }

  update(node) {
    // Guard against undefined type
    if (!node?.type || (this.node?.type && node.type !== this.node.type)) {
        console.warn('Update failed: invalid node type', {
            newType: node?.type,
            currentType: this.node?.type
        });
        return false;
    }
    
    this.node = node

    // Start elapsed timer on first update if not already started
    if (!this.startTime && node.attrs.current_step > 0) {
        this.startTime = new Date()
        this.startElapsedTimer()
    }

    // Safer progress calculation with bounds checking
    const currentStep = parseInt(node.attrs.current_step)
    const totalSteps = parseInt(node.attrs.total_steps)
    const progress = totalSteps > 0 
        ? Math.min(100, Math.round((currentStep / totalSteps) * 100))
        : 0

    // Now check for completion
    if (progress === 100 && !this.completed) {
        this.completed = true
        
        if (this.timeInterval) {
            clearInterval(this.timeInterval)
        }
        if (this.elapsedInterval) {
            clearInterval(this.elapsedInterval)
        }

        // Handle completion through ProseMirror - only update meta
        try {
            if (this.view?.state) {
                window.updateProgressNode(this.view, node.attrs.progress_id, {
                    meta: 'Completed',
                    current_step: node.attrs.total_steps  // Ensure we're at 100%
                })
            }
        } catch (error) {
            console.warn('Failed to update progress node on completion:', error)
        }

        // Update elapsed display one final time
        const elapsedSpan = this.dom.querySelector('.progress-elapsed')
        if (elapsedSpan) {
            const finalElapsed = Math.floor((new Date() - this.startTime) / 1000)
            const minutes = Math.floor(finalElapsed / 60)
            const seconds = finalElapsed % 60
            elapsedSpan.textContent = `Complete: ${minutes}:${seconds.toString().padStart(2, '0')}`
        }
    }

    // Update all the dynamic parts
    const countSpan = this.dom.querySelector('.progress-count')
    if (countSpan) {
      countSpan.textContent = `${node.attrs.current_step}/${node.attrs.total_steps}`
    }

    const fileDiv = this.dom.querySelector('.progress-file')
    if (fileDiv) {
      fileDiv.textContent = node.attrs.current_file
    }

    // Update progress bar with bounded value
    const progressBar = this.dom.querySelector('.progress-bar')
    if (progressBar) {
        progressBar.style.width = `${progress}%`
        
        // Color transitions using bounded progress value
        if (progress < 30) {
            progressBar.style.background = 'linear-gradient(90deg, #ff0000, #ff6b6b)'
        } else if (progress < 70) {
            progressBar.style.background = 'linear-gradient(90deg, #ffa500, #ffd700)'
        } else {
            progressBar.style.background = 'linear-gradient(90deg, #007bff, #00ff88)'
        }
    }

    const metaSpan = this.dom.querySelector('.progress-meta')
    if (metaSpan) {
        metaSpan.textContent = node.attrs.meta || ''
    }

    return true
  }

  destroy() {
    // Clear both intervals
    if (this.timeInterval) {
      clearInterval(this.timeInterval)
    }
    if (this.elapsedInterval) {
      clearInterval(this.elapsedInterval)
    }
  }

  stopEvent() {
    return true
  }
}

// ==========================================
// 2. TIPTAP EXTENSION DEFINITION
// This defines how the node works within Tiptap
// ==========================================
export const ProgressExtension = Node.create({
  name: 'progressIndicator',
  group: 'block',
  atom: true,

  // Add this to ensure proper node creation
  onCreate() {
    console.log('Progress node created:', this.type.name)
  },

  addAttributes() {
    return {
      progress_id: {
        default: 'default-progress-indicator'
      },
      current_step: {
        default: 0
      },
      total_steps: {
        default: 100
      },
      current_file: {
        default: ''
      },
      meta: {
        default: 'Progress'
      }
    }
  },

  parseHTML() {
    return [
      {
        tag: 'div[data-type="progress-indicator"]',
      },
    ]
  },

  renderHTML({ HTMLAttributes }) {
    return ['div', { 'data-type': 'progress-indicator', ...HTMLAttributes }, 0]
  },

  addNodeView() {
    return (node, view, getPos) => {
      return new ProgressView(node, view, getPos)
    }
  },
})

// ==========================================
// 3. GLOBAL UPDATE FUNCTION
// This lets you update the progress from anywhere
// ==========================================
window.updateProgressNode = (editor, progress_id, updates) => {
  const transaction = editor.state.tr

  editor.state.doc.descendants((node, pos) => {
    if (node.type.name === 'progressIndicator' && node.attrs.progress_id === progress_id) {
      const newAttrs = { ...node.attrs }
      
      if (updates.current_step !== undefined) {
        newAttrs.current_step = updates.current_step
      }
      if (updates.total_steps !== undefined) {
        newAttrs.total_steps = updates.total_steps
      }
      if (updates.current_file !== undefined) {
        newAttrs.current_file = updates.current_file
      }
      if (updates.meta !== undefined) {
        newAttrs.meta = updates.meta
      }

      transaction.setNodeMarkup(pos, null, newAttrs)
      return false // Stop traversing
    }
  })

  editor.view.dispatch(transaction)
}

// ==========================================
// 4. USAGE EXAMPLE
// This shows how to set up and use the extension
// ==========================================
/*
import { Editor } from '@tiptap/core'
import StarterKit from '@tiptap/starter-kit'
import { ProgressExtension } from './progressNode'

// Create the editor
const editor = new Editor({
  element: document.querySelector('.editor'),
  extensions: [
    StarterKit,
    ProgressExtension,
  ],
  content: {
    type: 'doc',
    content: [{
      type: 'progressIndicator',
      HTMLAttributes: {
        progress_id: 'chunk-progress-1',
        current_step: 0,
        total_steps: 300,
        current_file: 'document.pdf'
      }
    }]
  },
})

// Later, update the progress from anywhere:
window.updateProgressNode(editor, 'chunk-progress-1', {
  current_step: 21,
  current_file: 'new-document.pdf'
})
*/