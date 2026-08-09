import bus from '@/bus'

export const openNewDrawing = () => {
  bus.emit('ELEPHANT::open-excalidraw', {
    fileName: `excalidraw-${Date.now()}.png`,
    title: 'Excalidraw',
    saveMode: 'png',
    insertOnSave: false
  })
}
