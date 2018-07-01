import { canvases } from './canvas_store'
import {
  createTexManager,
  setTexDrawSize,
} from './tex_manager'

export {
  textBindFontFamily,
  textUnbindFontFamily,
  textSetFont,
  textGetWidth,
  textToTex,
} from './text'
export {
  imageLoadUrl,
  imageUnload,
  imageGetNaturalWidth,
  imageGetNaturalHeight,
  texFromImage,
} from './image'
export {
  texGetSize,
  texGetCount,
  texGetMaxDraws,
  texCreateEmpty,
  texCopy,
  texBindRenderingTarget,
  texUnbindRenderingTarget,
  texDelete,
  texDraw,
  texSetActiveTexture,
  texDrawEnd,
} from './tex_manager'

const initCanvas = function(canvas, gl) {
  gl.enable(gl.BLEND)
  gl.blendFunc(gl.ONE, gl.ONE_MINUS_SRC_ALPHA)
  gl.pixelStorei(gl.UNPACK_PREMULTIPLY_ALPHA_WEBGL, true)
  gl.viewport(0, 0, canvas.width, canvas.height)
  gl.clearColor(1.0, 1.0, 1.0, 0.0)
  // gl.enable(gl.DEPTH_TEST) // NOTE depth test disabled for simplexity
  // gl.clearDepth(0)
  gl.clear(gl.COLOR_BUFFER_BIT|gl.DEPTH_BUFFER_BIT)
}

export const bindCanvas = function(canvasIndex) {
  const elem = document.querySelector('canvas[glayout="' + canvasIndex + '"]')
  const canvasOption = {premultipliedAlpha: true}
  const ctx = elem.getContext('webgl', canvasOption) || elem.getContext('experimental-webgl', canvasOption)
  initCanvas(elem, ctx)
  canvases[canvasIndex] = {
    canvas: elem,
    ctx,
    texManager: createTexManager(ctx),
    texMap: [],
  }
}

export const unbindCanvas = function(canvasIndex) {
  canvases[canvasIndex] = null
  // TODO delete all textures and framebuffer etc in texManager
}

export const setCanvasSize = function(canvasIndex, w, h, pixelRatio) {
  const {canvas, ctx, texManager} = canvases[canvasIndex]
  canvas.style.width = w + 'px'
  canvas.style.height = h + 'px'
  canvas.width = w * pixelRatio
  canvas.height = h * pixelRatio
  setTexDrawSize(ctx, texManager, w, h, pixelRatio)
}

export const getDevicePixelRatio = function() {
  return window.devicePixelRatio
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
