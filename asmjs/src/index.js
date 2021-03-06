import { createBackgroundCanvas } from './webgl/canvas_store'
import * as externInterfaces from './extern_interfaces'

export * from './webgl/index'
export const extern = externInterfaces

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
  if (typeof __glayoutAsm__ === 'undefined') throw new Error('GLayout asm module not found. Initializing failed.')
  createBackgroundCanvas()
  console.log('[GLayout] [log] GLayout initialized.')
}

export const setLogLevelNum = function(num) {
  __glayoutAsm__._set_log_level_num(num)
  if (__glayoutAsm__._set_test_log_level_num) {
    __glayoutAsm__._set_test_log_level_num(num)
  }
}

export const consoleLogWithLevel = function(utf8Str, level) {
  const str = __glayoutAsm__.UTF8ToString(utf8Str)
  if (level === 3) console.error(str)
  else if (level === 2) console.warn(str)
  else if (level === 1) console.info(str)
  else if (level === -1) console.debug(str)
  else console.log(str)
}

export const loadTestCases = function() {
  __glayoutAsm__._load_test_cases()
  console.log('[GLayout] [log] GLayout test cases loaded.')
}

export const runTestCase = function(testCaseName) {
  var bufAddr = __glayoutAsm__._get_swap_buffer(4096)
  __glayoutAsm__.stringToUTF8(testCaseName, bufAddr, 4096)
  __glayoutAsm__._run_test_case(bufAddr)
}
