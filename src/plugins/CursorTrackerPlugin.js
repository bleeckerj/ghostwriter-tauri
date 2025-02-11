// 3. Create a plugin to handle cursor movement and item placement
const CursorTrackerPlugin = new Plugin({
  key: new PluginKey('cursorTracker'),
  
  view(editorView) {
    return {
      update: (view, prevState) => {
        const { selection } = view.state
        if (prevState && selection.eq(prevState.selection)) {
          return
        }
        
        // Remove any existing items
        const existingItems = document.querySelectorAll('.inline-action-item')
        existingItems.forEach(item => item.remove())
        
        // Get cursor coordinates
        const coords = view.coordsAtPos(selection.from)
        
        // Create and position the item
        const item = document.createElement('span')
        item.textContent = 'NFL'
        item.className = 'inline-action-item'
        item.style.position = 'absolute'
        item.style.left = `${coords.right}px`
        item.style.top = `${coords.top}px`
        
        document.body.appendChild(item)
      }
    }
  }
})