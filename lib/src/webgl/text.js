import { bgCanvas, fontFamilyMap } from './canvas_store'
import { texCreate } from './tex_manager'

export const textBindFontFamily = function(id, fontFamily) {
  fontFamilyMap[id] = __glayoutAsm__.UTF8ToString(fontFamily)
}

export const textUnbindFontFamily = function(id) {
  fontFamilyMap[id] = ''
}

export const textSetFont = function(fontSize, fontFamilyId, italic, bold) {
  const font = bgCanvas.font = (italic ? 'italic ' : '') + (bold ? 'bold ' : '') + fontSize + 'px ' + (fontFamilyMap[fontFamilyId] || 'sans-serif')
  bgCanvas.ctx.font = font
}

export const textGetWidth = function(text) {
  const ctx = bgCanvas.ctx
  return ctx.measureText(__glayoutAsm__.UTF8ToString(text)).width
}

export const textToTex = function(canvasIndex, texId, text, width, height) {
  const {canvas, ctx, font} = bgCanvas
  canvas.width = width
  canvas.height = height
  ctx.font = font
  ctx.textBaseline = 'middle'
  ctx.fillStyle = '#000'
  ctx.fillText(__glayoutAsm__.UTF8ToString(text), 0, Math.ceil(height / 2 + 1))
  texCreate(canvasIndex, bgCanvas.canvas, texId)
}
