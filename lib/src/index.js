import { createBackgroundCanvas } from './webgl/canvas_store'

export * from './webgl/index'

let animationFrameEnabled = false
let animationFrameScheduled = false

const animationFrameFn = function(timestamp) {
  if (!animationFrameEnabled) {
    animationFrameScheduled = false
    return
  }
  requestAnimationFrame(animationFrameFn)
  __glayoutAsm__._animation_frame(timestamp)
}

export const enableAnimationFrame = function() {
  animationFrameEnabled = true
  if (!animationFrameScheduled) {
    animationFrameScheduled = true
    requestAnimationFrame(animationFrameFn)
  }
}

export const disableAnimationFrame = function() {
  animationFrameEnabled = false
}

export const initLib = function() {
  if (typeof __glayoutAsm__ === 'undefined') throw new Error('Lanyout asm module not found. Initializing failed.')
  createBackgroundCanvas()
  console.log('Lanyout initialized.')
}
