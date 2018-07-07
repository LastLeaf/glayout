import { canvases } from './canvas_store'

import imgVs from './glsl/img.v.glsl'
import imgFs from './glsl/img.f.glsl'

const GL_DRAW_RECT_MAX = 65536 / 8
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

const generateIndexBufContent = function(buf) {
  for (let i = 0; i <= GL_DRAW_RECT_MAX; i++) {
    const base4 = i * 4
    const base6 = i * 6
    buf[base6] = base4
    buf[base6 + 1] = base4 + 1
    buf[base6 + 2] = base4 + 2
    buf[base6 + 3] = base4
    buf[base6 + 4] = base4 + 2
    buf[base6 + 5] = base4 + 3
  }
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

  // the element indices buffer
  const indexGLBuf = ctx.createBuffer()
  const indexBuf = new Uint16Array(6 * GL_DRAW_RECT_MAX)
  generateIndexBufContent(indexBuf)
  ctx.bindBuffer(ctx.ELEMENT_ARRAY_BUFFER, indexGLBuf)
  ctx.bufferData(ctx.ELEMENT_ARRAY_BUFFER, indexBuf, ctx.STATIC_DRAW)

  // the temp framebuffer and texture
  const tempFramebuffer = ctx.createFramebuffer()
  const tempTex = ctx.createTexture()
  ctx.bindTexture(ctx.TEXTURE_2D, tempTex)
  ctx.texParameteri(ctx.TEXTURE_2D, ctx.TEXTURE_MIN_FILTER, ctx.LINEAR)
  ctx.texParameteri(ctx.TEXTURE_2D, ctx.TEXTURE_WRAP_S, ctx.CLAMP_TO_EDGE)
  ctx.texParameteri(ctx.TEXTURE_2D, ctx.TEXTURE_WRAP_T, ctx.CLAMP_TO_EDGE)
  ctx.texImage2D(ctx.TEXTURE_2D, 0, ctx.RGBA, 256, 256, 0, ctx.RGBA, ctx.UNSIGNED_BYTE, null)
  ctx.bindTexture(ctx.TEXTURE_2D, null)

  // get other vars
  const uAreaSize = ctx.getUniformLocation(imgShaderProgram, 'uAreaSize')

  // bind default tex
  for (let i = 0; i < TEXTURE_MAX; i++) {
    ctx.activeTexture(ctx.TEXTURE0 + i)
    ctx.bindTexture(ctx.TEXTURE_2D, tempTex)
    const uTexI = ctx.getUniformLocation(imgShaderProgram, 'uTex' + i)
    ctx.uniform1i(uTexI, i)
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
    drawPosGLBuf,
    drawPosBuf,
    texIndexGLBuf,
    texIndexBuf,
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

export const texRewrite = function(canvasIndex, img, texId, left, top) {
  const {ctx, texManager, texMap} = canvases[canvasIndex]
  const tex = texId < 0 ? texManager.tempTex : texMap[texId]
  ctx.bindTexture(ctx.TEXTURE_2D, tex)
  ctx.texSubImage2D(ctx.TEXTURE_2D, 0, left, top, ctx.RGBA, ctx.UNSIGNED_BYTE, img)
  ctx.bindTexture(ctx.TEXTURE_2D, null)
}

export const texCopy = function(canvasIndex, destTexId, destLeft, destTop, srcLeft, srcTop, width, height) {
  const {ctx, texMap} = canvases[canvasIndex]
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
  ctx.uniform2f(texManager.uAreaSize, width, height)
  ctx.clearColor(0.0, 0.0, 0.0, 0.0)
  ctx.clearDepth(0)
  ctx.clear(ctx.COLOR_BUFFER_BIT|ctx.DEPTH_BUFFER_BIT)
}

export const texUnbindRenderingTarget = function(canvasIndex) {
  const {ctx, texManager} = canvases[canvasIndex]
  const {width, height, pixelRatio} = texManager
  ctx.bindFramebuffer(ctx.FRAMEBUFFER, null)
  ctx.useProgram(texManager.imgShaderProgram)
  ctx.viewport(0, 0, width * pixelRatio, height * pixelRatio)
  ctx.uniform2f(texManager.uAreaSize, width, height)
}

export const texCreateEmpty = function(canvasIndex, texId, width, height) {
  const {ctx, texMap} = canvases[canvasIndex]
  const tex = texMap[texId] = ctx.createTexture()
  ctx.bindTexture(ctx.TEXTURE_2D, tex)
  ctx.texParameteri(ctx.TEXTURE_2D, ctx.TEXTURE_MIN_FILTER, ctx.LINEAR)
  ctx.texParameteri(ctx.TEXTURE_2D, ctx.TEXTURE_WRAP_S, ctx.CLAMP_TO_EDGE)
  ctx.texParameteri(ctx.TEXTURE_2D, ctx.TEXTURE_WRAP_T, ctx.CLAMP_TO_EDGE)
  ctx.texImage2D(ctx.TEXTURE_2D, 0, ctx.RGBA, width, height, 0, ctx.RGBA, ctx.UNSIGNED_BYTE, null)
  texBindRenderingTarget(canvasIndex, texId, width, height)
  texUnbindRenderingTarget(canvasIndex)
  ctx.bindTexture(ctx.TEXTURE_2D, null)
}

export const texDelete = function(canvasIndex, texId) {
  const {ctx, texMap} = canvases[canvasIndex]
  const tex = texMap[texId]
  ctx.deleteTexture(tex)
  texMap[texId] = null
}

export const texDraw = function(canvasIndex, drawIndex, texShaderIndex, normalizedTexX, normalizedTexY, normalizedTexW, normalizedTexH, x, y, w, h) {
  const {texManager} = canvases[canvasIndex]
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
  texIndexBuf[drawIndex4 + 0] = texShaderIndex
  texIndexBuf[drawIndex4 + 1] = texShaderIndex
  texIndexBuf[drawIndex4 + 2] = texShaderIndex
  texIndexBuf[drawIndex4 + 3] = texShaderIndex
}

export const texSetActiveTexture = function(canvasIndex, texShaderIndex, texId) {
  const {ctx, texManager, texMap} = canvases[canvasIndex]
  ctx.activeTexture(ctx.TEXTURE0 + texShaderIndex)
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

  ctx.drawElements(ctx.TRIANGLES, drawCount * 6, ctx.UNSIGNED_SHORT, 0)
}

const texGetPixels = function(canvasIndex, left, top, width, height) {
  const {ctx} = canvases[canvasIndex]
  const ret = new window.Uint8Array(width * height * 4)
  ctx.readPixels(left, top, width, height, ctx.RGBA, ctx.UNSIGNED_BYTE, ret)
  return ret
}
