import { canvases, bgCanvas } from './canvas_store'

export const textBindFontFamily = function(canvasIndex, id, fontFamily) {
  const {fontFamilyMap} = canvases[canvasIndex]
  fontFamilyMap[id] = __glayoutAsm__.UTF8ToString(fontFamily)
}

export const textUnbindFontFamily = function(canvasIndex, id) {
  const {fontFamilyMap} = canvases[canvasIndex]
  fontFamilyMap[id] = ''
}

export const textSetFont = function(canvasIndex, fontSize, fontFamilyId) {
  const {fontFamilyMap} = canvases[canvasIndex]
  const ctx = bgCanvas.ctx
  ctx.font = fontSize + 'px ' + fontFamilyMap[fontFamilyId]
}

export const textGetWidth = function(text) {
  const ctx = bgCanvas.ctx
  return ctx.measureText(text).width
}

export const textDrawInCanvas = function(text, width, fontSize, fontFamilyId) {
  const ctx = bgCanvas.ctx
  const height = fontSize
  ctx.clearRect(0, 0, width, height)
  ctx.textBaseline = 'top'
  ctx.fillStyle = 'black'
  ctx.fillText(text, 0, 0)
  var imgData = this.elem = ctx.getImageData(0, 0, width, height)
  this.width = imgData.width
  this.height = imgData.height
}

export const textSaveImageData = function(canvasIndex, id, x, y, width, height) {
  const {textImageDataMap} = canvases[canvasIndex]
  const ctx = bgCanvas.ctx
  textImageDataMap[id] = ctx.getImageData(0, 0, width, height)
}

export const textRemoveImageData = function(canvasIndex, id) {
  const {textImageDataMap} = canvases[canvasIndex]
  textImageDataMap[id] = null
}
