export const canvases = []
export const bgCanvas = { canvas: null, ctx: null, font: '10px sans-serif' }
export const fontFamilyMap = []
export const imageElementMap = []

export const createBackgroundCanvas = function() {
  const elem = document.createElement('canvas')
  elem.width = 4096
  elem.height = 4096
  elem.style.transfrom = 'translateZ(0)'
  const ctx = elem.getContext('2d')
  bgCanvas.canvas = elem
  bgCanvas.ctx = ctx
  ctx.fillStyle = 'black'
  ctx.textAlign = 'left'
  ctx.textBaseline = 'top'
  document.body.appendChild(elem)
}
