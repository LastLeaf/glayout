import { bgCanvas, fontFamilyMap } from './canvas_store'
import { texRewrite } from './tex_manager'

export const textBindFontFamily = function(id, fontFamily) {
  fontFamilyMap[id] = __glayoutAsm__.UTF8ToString(fontFamily)
}

export const textUnbindFontFamily = function(id) {
  fontFamilyMap[id] = ''
}

export const textSetFont = function(fontSize, lineHeight, fontFamilyId, italic, bold) {
  const font = bgCanvas.font = (italic ? 'italic ' : '') + (bold ? 'bold ' : '') + fontSize + 'px/' + lineHeight + 'px ' + (fontFamilyMap[fontFamilyId] || 'sans-serif')
  bgCanvas.ctx.font = font
}

export const textGetWidth = function(text) {
  const ctx = bgCanvas.ctx
  return ctx.measureText(__glayoutAsm__.UTF8ToString(text)).width
}

export const textToTex = function(canvasIndex, texId, texLeft, texTop, text, width, height, lineHeight) {
  const {canvas, ctx, font} = bgCanvas
  canvas.width = width
  canvas.height = height
  ctx.font = font
  ctx.textBaseline = 'middle'
  ctx.fillStyle = '#000'
  const rows = __glayoutAsm__.UTF8ToString(text).split('\n')
  for (let i = 0; i < rows.length; i++) {
    ctx.fillText(rows[i], 0, lineHeight / 2 + lineHeight * i)
  }
  texRewrite(canvasIndex, bgCanvas.canvas, texId, texLeft, texTop)
}
