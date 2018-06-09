import { canvases } from './canvas_store'

import imgVs from './glsl/img.v.glsl'
import imgFs from './glsl/img.f.glsl'

const GL_DRAW_RECT_MAX = 65536

export const createTexManager = function(ctx) {
  const texSize = ctx.getParameter(ctx.MAX_TEXTURE_SIZE)
  const texCount = ctx.getParameter(ctx.MAX_TEXTURE_IMAGE_UNITS)

  // init img draw program
  const imgShaderProgram = ctx.createProgram()
  let shaderLog = ''
  let shader = ctx.createShader(ctx.VERTEX_SHADER)
  ctx.shaderSource(shader, imgVs)
  ctx.compileShader(shader)
  if(!ctx.getShaderParameter(shader, ctx.COMPILE_STATUS)) {
    shaderLog = ctx.getShaderInfoLog(shader)
    ctx.deleteShader(shader)
    throw new Error('Failed initializing WebGL vertex shader: ' + shaderLog)
  }
  ctx.attachShader(imgShaderProgram, shader)
  shader = ctx.createShader(ctx.FRAGMENT_SHADER)
  ctx.shaderSource(shader, imgFs)
  ctx.compileShader(shader)
  if(!ctx.getShaderParameter(shader, ctx.COMPILE_STATUS)) {
    shaderLog = ctx.getShaderInfoLog(shader)
    ctx.deleteShader(shader)
    throw new Error('Failed initializing WebGL fragment shader: ' + shaderLog)
  }
  ctx.attachShader(imgShaderProgram, shader)
  ctx.linkProgram(imgShaderProgram)
  if(!ctx.getProgramParameter(imgShaderProgram, ctx.LINK_STATUS)) {
    throw new Error('Failed initializing WebGL shader program.')
  }
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

  // tex id uniform
  for (let i = 0; i < 16; i++) {
    const uTexI = ctx.getUniformLocation(imgShaderProgram, 'uTex' + i)
    ctx.uniform1i(uTexI, i)
  }

  const texManager = {
    width: 1,
    height: 1,
    texSize,
    texCount,
    imgShaderProgram,
    texPosGLBuf,
    texPosBuf,
    drawPosGLBuf,
    drawPosBuf,
    texIndexGLBuf,
    texIndexBuf,
  }
  return texManager
}

export const setTexDrawSize = function(ctx, texManager, w, h) {
  texManager.width = w
  texManager.height = h
  const uAreaSize = ctx.getUniformLocation(texManager.imgShaderProgram, 'uAreaSize')
  ctx.uniform2f(uAreaSize, w, h)
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
  const {ctx, texMap} = canvases[canvasIndex]
  const tex = texMap[texId] = ctx.createTexture()
  ctx.bindTexture(ctx.TEXTURE_2D, tex)
  ctx.texParameteri(ctx.TEXTURE_2D, ctx.TEXTURE_MIN_FILTER, ctx.LINEAR)
  ctx.texParameteri(ctx.TEXTURE_2D, ctx.TEXTURE_WRAP_S, ctx.CLAMP_TO_EDGE)
  ctx.texParameteri(ctx.TEXTURE_2D, ctx.TEXTURE_WRAP_T, ctx.CLAMP_TO_EDGE)
  ctx.texImage2D(ctx.TEXTURE_2D, 0, ctx.RGBA, ctx.RGBA, ctx.UNSIGNED_BYTE, img)
  ctx.bindTexture(ctx.TEXTURE_2D, null)
}

export const texDelete = function(canvasIndex, texId) {
  const {ctx, texMap} = canvases[canvasIndex]
  const tex = texMap[texId]
  ctx.deleteTexture(tex)
  texMap[texId] = null
}

export const texDraw = function(canvasIndex, drawIndex, texId, texX, texY, texW, texH, x, y, w, h) {
  const {ctx, texManager, texMap} = canvases[canvasIndex]
  const {
    texPosBuf,
    drawPosBuf,
    texIndexBuf,
  } = texManager
  const drawIndex8 = drawIndex << 3
  const drawIndex4 = drawIndex << 2
  texPosBuf[drawIndex8 + 0] = texX
  texPosBuf[drawIndex8 + 1] = texY
  texPosBuf[drawIndex8 + 2] = texX
  texPosBuf[drawIndex8 + 3] = texY + texH
  texPosBuf[drawIndex8 + 4] = texX + texW
  texPosBuf[drawIndex8 + 5] = texY + texH
  texPosBuf[drawIndex8 + 6] = texX + texW
  texPosBuf[drawIndex8 + 7] = texY
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
  ctx.bindTexture(ctx.TEXTURE_2D, texMap[texId])
}

export const texDrawEnd = function(canvasIndex, drawCount) {
  const {ctx, texManager} = canvases[canvasIndex]
  const {
    imgShaderProgram,
    texPosGLBuf,
    texPosBuf,
    drawPosGLBuf,
    drawPosBuf,
    texIndexGLBuf,
    texIndexBuf,
  } = texManager

  ctx.useProgram(imgShaderProgram)

  ctx.bindBuffer(ctx.ARRAY_BUFFER, texPosGLBuf)
  ctx.bufferData(ctx.ARRAY_BUFFER, texPosBuf, ctx.DYNAMIC_DRAW)

  ctx.bindBuffer(ctx.ARRAY_BUFFER, drawPosGLBuf)
  ctx.bufferData(ctx.ARRAY_BUFFER, drawPosBuf, ctx.DYNAMIC_DRAW)

  ctx.bindBuffer(ctx.ARRAY_BUFFER, texIndexGLBuf)
  ctx.bufferData(ctx.ARRAY_BUFFER, texIndexBuf, ctx.DYNAMIC_DRAW)

  ctx.drawArrays(ctx.TRIANGLE_FAN, 0, drawCount << 2)
}
