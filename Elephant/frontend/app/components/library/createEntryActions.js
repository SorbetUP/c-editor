import bus from '@/bus'

export const openNewDrawing = () => {
  bus.emit('ELEPHANT::open-excalidraw', {
    fileName: 'drawing.png',
    title: 'Excalidraw',
    saveMode: 'png',
    insertOnSave: false,
    createNoteOnSave: true,
    askNameOnClose: true
  })
}
