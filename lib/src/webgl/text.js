import { bgCanvas, fontFamilyMap } from './canvas_store'
import { texCreate } from './tex_manager'

export const textBindFontFamily = function(id, fontFamily) {
  fontFamilyMap[id] = __glayoutAsm__.UTF8ToString(fontFamily)
}

export const textUnbindFontFamily = function(id) {
  fontFamilyMap[id] = ''
}

export const textSetFont = function(fontSize, fontFamilyId, italic, bold) {
  const ctx = bgCanvas.ctx
  ctx.font = (italic ? 'italic ' : '') + (bold ? 'bold ' : '') + fontSize + 'px ' + (fontFamilyMap[fontFamilyId] || 'sans-serif')
}

export const textGetWidth = function(text) {
  const ctx = bgCanvas.ctx
  return ctx.measureText(__glayoutAsm__.UTF8ToString(text)).width
}

export const textDrawInCanvas = function(text, width, height) {
  const ctx = bgCanvas.ctx
  ctx.clearRect(0, 0, width, height)
  ctx.textBaseline = 'middle'
  ctx.fillStyle = 'black'
  ctx.fillText(__glayoutAsm__.UTF8ToString(text), 0, height / 2)
}

export const texFromText = function(canvasIndex, texId, x, y, width, height) {
  const bgCtx = bgCanvas.ctx
  const imgData = bgCtx.getImageData(x, y, width, height)
  texCreate(canvasIndex, imgData, texId)
}
