import { Extension } from '@tiptap/core'
import { createVirtualCursor } from 'prosemirror-virtual-cursor';

export const VirtualCursor = Extension.create({
  name: 'virtualCursor',

  addProseMirrorPlugins() {
    return [
      createVirtualCursor({
        className: 'virtual-cursor',
        color: '#3b82f6', // Tailwind blue-500
        thickness: 2,
        height: '1.2em'
      })
    ]
  }
})