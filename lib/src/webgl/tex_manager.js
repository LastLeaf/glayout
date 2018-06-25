import { canvases } from './canvas_store'

import imgVs from './glsl/img.v.glsl'
import imgFs from './glsl/img.f.glsl'

const GL_DRAW_RECT_MAX = 65536
const TEXTURE_MAX = 16

const createProgram = function(ctx, vs, fs) {
  const shaderProgram = ctx.createProgram()
  let shaderLog = ''
  let shader = ctx.createShader(ctx.VERTEX_SHADER)
  ctx.shaderSource(shader, vs)
  ctx.compileShader(shader)
  if(!ctx.getShaderParameter(shader, ctx.COMPILE_STATUS)) {
    shaderLog = ctx.getShaderInfoLog(shader)
    ctx.deleteShader(shader)
    throw new Error('Failed initializing WebGL vertex shader: ' + shaderLog)
  }
  ctx.attachShader(shaderProgram, shader)
  shader = ctx.createShader(ctx.FRAGMENT_SHADER)
  ctx.shaderSource(shader, fs)
  ctx.compileShader(shader)
  if(!ctx.getShaderParameter(shader, ctx.COMPILE_STATUS)) {
    shaderLog = ctx.getShaderInfoLog(shader)
    ctx.deleteShader(shader)
    throw new Error('Failed initializing WebGL fragment shader: ' + shaderLog)
  }
  ctx.attachShader(shaderProgram, shader)
  ctx.linkProgram(shaderProgram)
  if(!ctx.getProgramParameter(shaderProgram, ctx.LINK_STATUS)) {
    throw new Error('Failed initializing WebGL shader program.')
  }
  return shaderProgram
}

export const createTexManager = function(ctx) {
  const texSize = ctx.getParameter(ctx.MAX_TEXTURE_SIZE)
  const texCount = ctx.getParameter(ctx.MAX_TEXTURE_IMAGE_UNITS)

  const imgShaderProgram = createProgram(ctx, imgVs, imgFs)
  ctx.useProgram(imgShaderProgram)

  // the texture position buffer
  const texPosGLBuf = ctx.createBuffer()
  const texPosBuf = new Float32Array(8 * GL_DRAW_RECT_MAX)
  ctx.bindBuffer(ctx.ARRAY_BUFFER, texPosGLBuf)
  ctx.bufferData(ctx.ARRAY_BUFFER, texPosBuf, ctx.DYNAMIC_DRAW)
  const aTexPos = ctx.getAttribLocation(imgShaderProgram, 'aTexPos')
  ctx.enableVertexAttribArray(aTexPos)
  ctx.vertexAttribPointer(aTexPos, 2, ctx.FLOAT, false, 0, 0)

  // the draw position buffer
  const drawPosGLBuf = ctx.createBuffer()
  const drawPosBuf = new Float32Array(8 * GL_DRAW_RECT_MAX)
  ctx.bindBuffer(ctx.ARRAY_BUFFER, drawPosGLBuf)
  ctx.bufferData(ctx.ARRAY_BUFFER, drawPosBuf, ctx.DYNAMIC_DRAW)
  const aDrawPos = ctx.getAttribLocation(imgShaderProgram, 'aDrawPos')
  ctx.enableVertexAttribArray(aDrawPos)
  ctx.vertexAttribPointer(aDrawPos, 2, ctx.FLOAT, false, 0, 0)

  // the texture index buffer
  const texIndexGLBuf = ctx.createBuffer()
  const texIndexBuf = new Float32Array(4 * GL_DRAW_RECT_MAX)
  ctx.bindBuffer(ctx.ARRAY_BUFFER, texIndexGLBuf)
  ctx.bufferData(ctx.ARRAY_BUFFER, texIndexBuf, ctx.DYNAMIC_DRAW)
  const aTexIndex = ctx.getAttribLocation(imgShaderProgram, 'aTexIndex')
  ctx.enableVertexAttribArray(aTexIndex)
  ctx.vertexAttribPointer(aTexIndex, 1, ctx.FLOAT, false, 0, 0)

  // the temp framebuffer and texture
  const tempFramebuffer = ctx.createFramebuffer()
  const tempTex = ctx.createTexture()
  const uAreaSize = ctx.getUniformLocation(imgShaderProgram, 'uAreaSize')

  // tex id uniform
  for (let i = 0; i < TEXTURE_MAX; i++) {
    const uTexI = ctx.getUniformLocation(imgShaderProgram, 'uTex' + i)
    ctx.uniform1i(uTexI, i)
    ctx.activeTexture(ctx.TEXTURE0 + i)
    ctx.bindTexture(ctx.TEXTURE_2D, tempTex)
  }

  const texManager = {
    width: 1,
    height: 1,
    pixelRatio: 1,
    uAreaSize,
    texSize,
    texCount,
    imgShaderProgram,
    texPosGLBuf,
    texPosBuf,
    aTexPos,
    drawPosGLBuf,
    drawPosBuf,
    aDrawPos,
    texIndexGLBuf,
    texIndexBuf,
    aTexIndex,
    tempFramebuffer,
    tempTex,
  }
  return texManager
}

export const setTexDrawSize = function(ctx, texManager, w, h, pixelRatio) {
  texManager.width = w
  texManager.height = h
  texManager.pixelRatio = pixelRatio
  ctx.viewport(0, 0, w * pixelRatio, h * pixelRatio)
  ctx.uniform2f(texManager.uAreaSize, w, h)
}

export const texGetSize = function(canvasIndex) {
  const {texManager} = canvases[canvasIndex]
  return texManager.texSize
}

export const texGetCount = function(canvasIndex) {
  const {texManager} = canvases[canvasIndex]
  return texManager.texCount
}

export const texGetMaxDraws = function() {
  return GL_DRAW_RECT_MAX
}

export const texCreate = function(canvasIndex, img, texId) {
  const {ctx, texManager, texMap} = canvases[canvasIndex]
  const tex = texId < 0 ? texManager.tempTex : (texMap[texId] = ctx.createTexture())
  ctx.bindTexture(ctx.TEXTURE_2D, tex)
  ctx.texParameteri(ctx.TEXTURE_2D, ctx.TEXTURE_MIN_FILTER, ctx.LINEAR)
  ctx.texParameteri(ctx.TEXTURE_2D, ctx.TEXTURE_WRAP_S, ctx.CLAMP_TO_EDGE)
  ctx.texParameteri(ctx.TEXTURE_2D, ctx.TEXTURE_WRAP_T, ctx.CLAMP_TO_EDGE)
  ctx.texImage2D(ctx.TEXTURE_2D, 0, ctx.RGBA, ctx.RGBA, ctx.UNSIGNED_BYTE, img)
  ctx.bindTexture(ctx.TEXTURE_2D, null)
}

export const texCreateEmpty = function(canvasIndex, texId, width, height) {
  const {ctx, texMap} = canvases[canvasIndex]
  const tex = texMap[texId] = ctx.createTexture()
  ctx.bindTexture(ctx.TEXTURE_2D, tex)
  ctx.texParameteri(ctx.TEXTURE_2D, ctx.TEXTURE_MIN_FILTER, ctx.LINEAR)
  ctx.texParameteri(ctx.TEXTURE_2D, ctx.TEXTURE_WRAP_S, ctx.CLAMP_TO_EDGE)
  ctx.texParameteri(ctx.TEXTURE_2D, ctx.TEXTURE_WRAP_T, ctx.CLAMP_TO_EDGE)
  ctx.texImage2D(ctx.TEXTURE_2D, 0, ctx.RGBA, width, height, 0, ctx.RGBA, ctx.UNSIGNED_BYTE, null)
  ctx.bindTexture(ctx.TEXTURE_2D, null)
}

export const texCopy = function(canvasIndex, destTexId, destLeft, destTop, srcLeft, srcTop, width, height) {
  const {ctx, texManager, texMap} = canvases[canvasIndex]
  ctx.bindTexture(ctx.TEXTURE_2D, texMap[destTexId])
  ctx.copyTexSubImage2D(ctx.TEXTURE_2D, 0, destLeft, destTop, srcLeft, srcTop, width, height)
  ctx.bindTexture(ctx.TEXTURE_2D, null)
}

export const texBindRenderingTarget = function(canvasIndex, texId, width, height) {
  const {ctx, texManager, texMap} = canvases[canvasIndex]
  ctx.bindFramebuffer(ctx.FRAMEBUFFER, texManager.tempFramebuffer)
  ctx.framebufferTexture2D(ctx.FRAMEBUFFER, ctx.COLOR_ATTACHMENT0, ctx.TEXTURE_2D, texId < 0 ? texManager.tempTex : texMap[texId], 0)
  ctx.useProgram(texManager.imgShaderProgram)
  ctx.viewport(0, 0, width, height)
  // ctx.enable(ctx.BLEND)
  // ctx.blendFunc(ctx.ONE, ctx.ONE_MINUS_SRC_ALPHA)
  ctx.uniform2f(texManager.uAreaSize, width, height)
  // ctx.clearColor(1.0, 1.0, 1.0, 0.0)
  // ctx.clearDepth(0)
  // ctx.clear(ctx.COLOR_BUFFER_BIT|ctx.DEPTH_BUFFER_BIT)
}

export const texUnbindRenderingTarget = function(canvasIndex) {
  const {ctx, texManager} = canvases[canvasIndex]
  const {width, height, pixelRatio} = texManager
  ctx.bindFramebuffer(ctx.FRAMEBUFFER, null)
  ctx.useProgram(texManager.imgShaderProgram)
  ctx.viewport(0, 0, width * pixelRatio, height * pixelRatio)
  // ctx.enable(ctx.BLEND)
  // ctx.blendFunc(ctx.ONE, ctx.ONE_MINUS_SRC_ALPHA)
  ctx.uniform2f(texManager.uAreaSize, width, height)
}

export const texDelete = function(canvasIndex, texId) {
  const {ctx, texMap} = canvases[canvasIndex]
  const tex = texMap[texId]
  ctx.deleteTexture(tex)
  texMap[texId] = null
}

export const texDraw = function(canvasIndex, drawIndex, texId, normalizedTexX, normalizedTexY, normalizedTexW, normalizedTexH, x, y, w, h) {
  const {ctx, texManager, texMap} = canvases[canvasIndex]
  const {
    texPosBuf,
    drawPosBuf,
    texIndexBuf,
  } = texManager
  const drawIndex8 = drawIndex << 3
  const drawIndex4 = drawIndex << 2
  texPosBuf[drawIndex8 + 0] = normalizedTexX
  texPosBuf[drawIndex8 + 1] = normalizedTexY
  texPosBuf[drawIndex8 + 2] = normalizedTexX
  texPosBuf[drawIndex8 + 3] = normalizedTexY + normalizedTexH
  texPosBuf[drawIndex8 + 4] = normalizedTexX + normalizedTexW
  texPosBuf[drawIndex8 + 5] = normalizedTexY + normalizedTexH
  texPosBuf[drawIndex8 + 6] = normalizedTexX + normalizedTexW
  texPosBuf[drawIndex8 + 7] = normalizedTexY
  drawPosBuf[drawIndex8 + 0] = x
  drawPosBuf[drawIndex8 + 1] = y
  drawPosBuf[drawIndex8 + 2] = x
  drawPosBuf[drawIndex8 + 3] = y + h
  drawPosBuf[drawIndex8 + 4] = x + w
  drawPosBuf[drawIndex8 + 5] = y + h
  drawPosBuf[drawIndex8 + 6] = x + w
  drawPosBuf[drawIndex8 + 7] = y
  texIndexBuf[drawIndex4 + 0] = drawIndex
  texIndexBuf[drawIndex4 + 1] = drawIndex
  texIndexBuf[drawIndex4 + 2] = drawIndex
  texIndexBuf[drawIndex4 + 3] = drawIndex
  ctx.activeTexture(ctx.TEXTURE0 + drawIndex)
  ctx.bindTexture(ctx.TEXTURE_2D, texId < 0 ? texManager.tempTex : texMap[texId])
}

export const texDrawEnd = function(canvasIndex, drawCount) {
  const {ctx, texManager} = canvases[canvasIndex]
  const {
    texPosGLBuf,
    texPosBuf,
    drawPosGLBuf,
    drawPosBuf,
    texIndexGLBuf,
    texIndexBuf,
  } = texManager

  ctx.bindBuffer(ctx.ARRAY_BUFFER, texPosGLBuf)
  ctx.bufferData(ctx.ARRAY_BUFFER, texPosBuf, ctx.DYNAMIC_DRAW)

  ctx.bindBuffer(ctx.ARRAY_BUFFER, drawPosGLBuf)
  ctx.bufferData(ctx.ARRAY_BUFFER, drawPosBuf, ctx.DYNAMIC_DRAW)

  ctx.bindBuffer(ctx.ARRAY_BUFFER, texIndexGLBuf)
  ctx.bufferData(ctx.ARRAY_BUFFER, texIndexBuf, ctx.DYNAMIC_DRAW)

  ctx.drawArrays(ctx.TRIANGLE_FAN, 0, drawCount << 2)
}

const texGetPixels = function(canvasIndex, left, top, width, height) {
  const {ctx} = canvases[canvasIndex]
  const ret = new window.Uint8Array(width * height * 4)
  ctx.readPixels(left, top, width, height, ctx.RGBA, ctx.UNSIGNED_BYTE, ret)
  return ret
}
