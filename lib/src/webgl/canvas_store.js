export const canvases = {}
export const bgCanvas = { canvas: null, ctx: null }

export const createBackgroundCanvas = function() {
  const elem = document.createElement('canvas')
  elem.width = 4096
  elem.height = 4096
  const ctx = elem.getContext('2d')
  bgCanvas.canvas = elem
  bgCanvas.ctx = ctx
}
