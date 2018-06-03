import { canvases } from './canvas_store'
import { createTexManager, setTexDrawSize } from './tex_manager'

export {
  textBindFontFamily,
  textUnbindFontFamily,
  textSetFont,
  textGetWidth,
  textDrawInCanvas,
  textSaveImageData,
  textRemoveImageData,
} from './text'
export {
  imageLoadUrl,
  imageUnload,
} from './image'
export {
  texGetSize,
  texGetCount,
  texGetMaxDraws,
  texSetText,
  texSetImage,
  texDraw,
  texDrawEnd,
} from './tex_manager'

const initCanvas = function(canvas, gl) {
  gl.enable(gl.BLEND)
  gl.enable(gl.DEPTH_TEST)
  gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA)
  gl.viewport(0, 0, canvas.width, canvas.height)
  gl.clearColor(1.0, 1.0, 1.0, 0.0)
  gl.clearDepth(0)
  gl.clear(gl.COLOR_BUFFER_BIT|gl.DEPTH_BUFFER_BIT)
}

export const bindCanvas = function(canvasIndex) {
  const elem = document.querySelector('canvas[glayout="' + canvasIndex + '"]')
  const ctx = elem.getContext('webgl') || elem.getContext('experimental-webgl')
  initCanvas(elem, ctx)
  canvases[canvasIndex] = {
    canvas: elem,
    ctx,
    texManager: createTexManager(elem, ctx),
    fontFamilyMap: [],
    textImageDataMap: [],
    imageElementMap: [],
  }
}

export const unbindCanvas = function(canvasIndex) {
  delete canvases[canvasIndex]
}

export const setCanvasSize = function(canvasIndex, w, h) {
  const {canvas, ctx, texManager} = canvases[canvasIndex]
  canvas.width = w
  canvas.height = h
  setTexDrawSize(ctx, texManager, w, h)
  ctx.viewport(0, 0, w, h)
}

export const setClearColor = function(canvasIndex, r, g, b, a) {
  const {ctx} = canvases[canvasIndex]
  ctx.clearColor(r, g, b, a)
}

export const clear = function(canvasIndex) {
  const {ctx} = canvases[canvasIndex]
  ctx.clear(ctx.COLOR_BUFFER_BIT | ctx.DEPTH_BUFFER_BIT)
}

export const timeout = function(ms, cbPtr) {
  setTimeout(() => {
    __glayoutAsm__._callback(cbPtr, Date.now())
  }, ms)
}
