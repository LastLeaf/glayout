import { canvases } from './canvas_store'

import imgVs from './glsl/img.v.glsl'
import imgFs from './glsl/img.f.glsl'

const GL_DRAW_RECT_MAX = 65536

export const createTexManager = function(canvas, ctx) {
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

  // the texture position buffer
  const texIndexGLBuf = ctx.createBuffer()
  const texIndexBuf = new Float32Array(GL_DRAW_RECT_MAX)
  ctx.bindBuffer(ctx.ARRAY_BUFFER, texIndexGLBuf)
  ctx.bufferData(ctx.ARRAY_BUFFER, texIndexBuf, ctx.DYNAMIC_DRAW)
  const aTexIndex = ctx.getAttribLocation(imgShaderProgram, 'aTexIndex')
  ctx.enableVertexAttribArray(aTexIndex)
  ctx.vertexAttribPointer(aTexIndex, 2, ctx.FLOAT, false, 0, 0)

  // create textures
  const textures = []
  for (let i = 0; i < texCount; i++) {
    textures.push(ctx.createTexture())
    ctx.activeTexture(ctx['TEXTURE' + i])
    ctx.bindTexture(ctx.TEXTURE_2D, textures[i])
  }

  return {
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
    textures,
  }
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

const setTex = function(canvasIndex, img, texIndex, texX, texY, texW, texH){
  const {ctx, texManager} = canvases[canvasIndex]
  const {textures} = texManager
  ctx.bindTexture(ctx.TEXTURE_2D, textures[texIndex])
  ctx.texParameteri(ctx.TEXTURE_2D, ctx.TEXTURE_MIN_FILTER, ctx.LINEAR)
  ctx.texParameteri(ctx.TEXTURE_2D, ctx.TEXTURE_WRAP_S, ctx.CLAMP_TO_EDGE)
  ctx.texParameteri(ctx.TEXTURE_2D, ctx.TEXTURE_WRAP_T, ctx.CLAMP_TO_EDGE)
  // ctx.texSubImage2D(ctx.TEXTURE_2D, 0, texX, texY, ctx.RGBA, ctx.UNSIGNED_BYTE, img)
  ctx.texImage2D(ctx.TEXTURE_2D, 0, ctx.RGBA, ctx.RGBA, ctx.UNSIGNED_BYTE, img) // TODO
  ctx.bindTexture(ctx.TEXTURE_2D, null)
}

export const texDraw = function(canvasIndex, drawIndex, texIndex, texX, texY, texW, texH, x, y, w, h) {
  // console.info('DRAW', canvasIndex, drawIndex, texIndex, texX, texY, texW, texH, x, y, w, h)
  const {texManager} = canvases[canvasIndex]
  const {
    texPosBuf,
    drawPosBuf,
    texIndexBuf,
  } = texManager
  const drawIndex8 = drawIndex * 8
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
  texIndexBuf[drawIndex] = texIndex
}

export const texDrawEnd = function(canvasIndex) {
  // console.info('DRAW_END', canvasIndex)
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
  ctx.drawArrays(ctx.TRIANGLE_FAN, 0, 4)
}

export const texSetText = function(canvasIndex, id, texIndex, texX, texY, texWidth, texHeight) {
  const {textImageDataMap} = canvases[canvasIndex]
  const imageData = textImageDataMap[id]
  setTex(canvasIndex, imageData, texIndex, texX, texY, texWidth, texHeight)
}

export const texSetImage = function(canvasIndex, id, texIndex, texX, texY, texWidth, texHeight) {
  const {imageElementMap} = canvases[canvasIndex]
  const imgElem = imageElementMap[id]
  setTex(canvasIndex, imgElem, texIndex, texX, texY, texWidth, texHeight)
}
