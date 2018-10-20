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
  texSetDrawState,
} from './tex_manager'

const IS_TOUCH_DEVICE = typeof window !== 'undefined' && typeof document !== 'undefined' && ('ontouchstart' in window || (navigator.msMaxTouchPoints > 0))

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
  // TODO unbind all event listeners
}

export const setCanvasSize = function(canvasIndex, w, h, pixelRatio, updateLogicalSize) {
  const {canvas, ctx, texManager} = canvases[canvasIndex]
  if (updateLogicalSize) {
    canvas.style.width = w + 'px'
    canvas.style.height = h + 'px'
  }
  canvas.width = w * pixelRatio
  canvas.height = h * pixelRatio
  setTexDrawSize(ctx, texManager, w, h, pixelRatio)
}
export const getCanvasWidth = function(canvasIndex) {
  const {canvas} = canvases[canvasIndex]
  return Math.floor(canvas.width / window.devicePixelRatio)
}
export const getCanvasHeight = function(canvasIndex) {
  const {canvas} = canvases[canvasIndex]
  return Math.floor(canvas.height / window.devicePixelRatio)
}
// eslint-disable-next-line no-unused-vars
export const getDevicePixelRatio = function(canvasIndex) {
  return window.devicePixelRatio
}

export const setClearColor = function(canvasIndex, r, g, b, a) {
  const {ctx} = canvases[canvasIndex]
  ctx.clearColor(r, g, b, a)
}

export const clear = function(canvasIndex) {
  const {ctx} = canvases[canvasIndex]
  ctx.clear(ctx.COLOR_BUFFER_BIT)
}

// window size utils
let windowSizeListener = 0;
if (typeof window !== 'undefined' && typeof document !== 'undefined') {
  window.addEventListener('resize', function() {
    if (windowSizeListener) {
      __glayoutAsm__._callback(windowSizeListener, 0, 0, 0, 0)
    }
  }, false)
}

// timing utils
export const timeout = function(ms, cbPtr) {
  setTimeout(() => {
    __glayoutAsm__._callback(cbPtr, 0, 0, 0, 0)
  }, ms)
}

// mouse/touch utils
let inTouchingProgress = false
export const bindTouchEvents = function(canvasIndex, cbPtr) {
  const {canvas} = canvases[canvasIndex]
  if (IS_TOUCH_DEVICE) {
    canvas.addEventListener('touchstart', function(e) {
      if (inTouchingProgress) return
      inTouchingProgress = true
      const rect = canvas.getBoundingClientRect()
      __glayoutAsm__._callback(cbPtr, 1, e.touches[0].clientX - rect.left, e.touches[0].clientY - rect.top, 0)
    })
    canvas.addEventListener('touchmove', function(e) {
      if (!inTouchingProgress) return
      const rect = canvas.getBoundingClientRect()
      __glayoutAsm__._callback(cbPtr, 2, e.touches[0].clientX - rect.left, e.touches[0].clientY - rect.top, 0)
    })
    canvas.addEventListener('touchend', function(e) {
      if (!inTouchingProgress) return
      inTouchingProgress = false
      const rect = canvas.getBoundingClientRect()
      __glayoutAsm__._callback(cbPtr, 3, e.changedTouches[0].clientX - rect.left, e.changedTouches[0].clientY - rect.top, 0)
    })
    canvas.addEventListener('touchcancel', function(e) {
      if (!inTouchingProgress) return
      inTouchingProgress = false
      const rect = canvas.getBoundingClientRect()
      __glayoutAsm__._callback(cbPtr, 4, e.changedTouches[0].clientX - rect.left, e.changedTouches[0].clientY - rect.top, 0)
    })
  } else {
    canvas.addEventListener('mousedown', function(e) {
      if (inTouchingProgress) return
      inTouchingProgress = true
      const rect = canvas.getBoundingClientRect()
      __glayoutAsm__._callback(cbPtr, 1, e.clientX - rect.left, e.clientY - rect.top, 0)
    })
    canvas.addEventListener('mousemove', function(e) {
      const rect = canvas.getBoundingClientRect()
      __glayoutAsm__._callback(cbPtr, inTouchingProgress ? 2 : 5, e.clientX - rect.left, e.clientY - rect.top, 0)
    })
    canvas.addEventListener('mouseup', function(e) {
      if (!inTouchingProgress) return
      inTouchingProgress = false
      const rect = canvas.getBoundingClientRect()
      __glayoutAsm__._callback(cbPtr, 3, e.clientX - rect.left, e.clientY - rect.top, 0)
    })
    canvas.addEventListener('mouseout', function(e) {
      if (!inTouchingProgress) return
      inTouchingProgress = false
      const rect = canvas.getBoundingClientRect()
      __glayoutAsm__._callback(cbPtr, 4, e.clientX - rect.left, e.clientY - rect.top, 0)
    })
  }
}

const convertKeyEvent = function(e) {
  return (e.shiftKey ? 8 : 0) +
    (e.ctrlKey ? 4 : 0) +
    (e.altKey ? 2 : 0) +
    (e.metaKey ? 1 : 0)
}
export const bindKeyboardEvents = function(canvasIndex, cbPtr) {
  window.addEventListener('keydown', function(e) {
    __glayoutAsm__._callback(cbPtr, 1, e.keyCode, e.charCode, convertKeyEvent(e))
  })
  window.addEventListener('keypress', function(e) {
    __glayoutAsm__._callback(cbPtr, 2, e.keyCode, e.charCode, convertKeyEvent(e))
  })
  window.addEventListener('keyup', function(e) {
    __glayoutAsm__._callback(cbPtr, 3, e.keyCode, e.charCode, convertKeyEvent(e))
  })
}

export const bindCanvasSizeChange = function(canvasIndex, cbPtr) {
  const {canvas} = canvases[canvasIndex]
  const computedStyle = window.getComputedStyle(canvas)
  let prevSize = { w: parseFloat(computedStyle.width), h: parseFloat(computedStyle.height), dpi: window.devicePixelRatio }
  setInterval(function() {
    const computedStyle = window.getComputedStyle(canvas)
    const size = { w: parseFloat(computedStyle.width), h: parseFloat(computedStyle.height), dpi: window.devicePixelRatio }
    const needCb = (prevSize.w !== size.w || prevSize.h !== size.h || prevSize.dpi != size.dpi)
    prevSize = size
    if (needCb) __glayoutAsm__._callback(cbPtr, size.w, size.h, size.dpi * 100000000, 0)
  }, 500)
}
