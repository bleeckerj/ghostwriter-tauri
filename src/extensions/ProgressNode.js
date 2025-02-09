// ProgressNode.js

import { Node } from '@tiptap/core'

// ==========================================
// 1. NODE VIEW CLASS
// This handles the actual DOM rendering and updates
// ==========================================
class ProgressView {
  constructor(node, view, getPos) {
    this.node = node
    this.view = view
    this.getPos = getPos
    this.dom = this.createDOM()
    this.contentDOM = null

    this.startTimeUpdate()
  }

  createDOM() {
    const container = document.createElement('div')
    container.classList.add('progress-indicator')
    container.setAttribute('data-progress-id', this.node.attrs.progressId)

    // Stats container (progress count + time)
    const statsDiv = document.createElement('div')
    statsDiv.classList.add('progress-stats')

    // Progress count
    const countSpan = document.createElement('span')
    countSpan.classList.add('progress-count')
    countSpan.textContent = `${this.node.attrs.currentStep}/${this.node.attrs.totalSteps}`

    // Time
    const timeSpan = document.createElement('span')
    timeSpan.classList.add('progress-time')
    timeSpan.textContent = new Date().toLocaleTimeString()

    statsDiv.appendChild(countSpan)
    statsDiv.appendChild(timeSpan)

    // File name
    const fileDiv = document.createElement('div')
    fileDiv.classList.add('progress-file')
    fileDiv.textContent = this.node.attrs.currentFile

    // Progress bar
    const progressBarContainer = document.createElement('div')
    progressBarContainer.classList.add('progress-bar-container')
    progressBarContainer.style.width = '100%'
    progressBarContainer.style.height = '4px'
    progressBarContainer.style.backgroundColor = '#eee'

    const progressBar = document.createElement('div')
    progressBar.classList.add('progress-bar')
    progressBar.style.width = `${(this.node.attrs.currentStep / this.node.attrs.totalSteps) * 100}%`
    progressBar.style.height = '100%'
    progressBar.style.backgroundColor = '#007bff'
    progressBar.style.transition = 'width 0.3s ease'

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

  update(node) {
    if (node.type !== this.node.type) return false
    this.node = node

    // Update all the dynamic parts
    const countSpan = this.dom.querySelector('.progress-count')
    if (countSpan) {
      countSpan.textContent = `${node.attrs.currentStep}/${node.attrs.totalSteps}`
    }

    const fileDiv = this.dom.querySelector('.progress-file')
    if (fileDiv) {
      fileDiv.textContent = node.attrs.currentFile
    }

    const progressBar = this.dom.querySelector('.progress-bar')
    if (progressBar) {
      progressBar.style.width = `${(node.attrs.currentStep / node.attrs.totalSteps) * 100}%`
    }

    return true
  }

  destroy() {
    if (this.timeInterval) {
      clearInterval(this.timeInterval)
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
  
  addAttributes() {
    return {
      progressId: {
        default: null,
      },
      currentStep: {
        default: 0,
      },
      totalSteps: {
        default: 0,
      },
      currentFile: {
        default: '',
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
window.updateProgressNode = (editor, progressId, updates) => {
  const transaction = editor.state.tr

  editor.state.doc.descendants((node, pos) => {
    if (node.type.name === 'progressIndicator' && node.attrs.progressId === progressId) {
      const newAttrs = { ...node.attrs }
      
      if (updates.currentStep !== undefined) {
        newAttrs.currentStep = updates.currentStep
      }
      if (updates.totalSteps !== undefined) {
        newAttrs.totalSteps = updates.totalSteps
      }
      if (updates.currentFile !== undefined) {
        newAttrs.currentFile = updates.currentFile
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
      attrs: {
        progressId: 'chunk-progress-1',
        currentStep: 0,
        totalSteps: 300,
        currentFile: 'document.pdf'
      }
    }]
  },
})

// Later, update the progress from anywhere:
window.updateProgressNode(editor, 'chunk-progress-1', {
  currentStep: 21,
  currentFile: 'new-document.pdf'
})
*/