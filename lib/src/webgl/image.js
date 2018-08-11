import { imageElementMap } from './canvas_store'
import { texCreate } from './tex_manager'

export const imageLoadUrl = function(imgId, url, cbPtr) {
  const imgElem = document.createElement('img')
  imgElem.onload = function() {
    imageElementMap[imgId] = imgElem
    __glayoutAsm__._callback(cbPtr, 0, 0, 0, 0)
  }
  imgElem.onerror = imgElem.onabort = function(){
    __glayoutAsm__._callback(cbPtr, -1, 0, 0, 0)
  }
  imgElem.src = __glayoutAsm__.UTF8ToString(url)
}

export const imageUnload = function(imgId) {
  delete imageElementMap[imgId]
}

export const imageGetNaturalWidth = function(imgId) {
  return imageElementMap[imgId].naturalWidth
}

export const imageGetNaturalHeight = function(imgId) {
  return imageElementMap[imgId].naturalHeight
}

export const texFromImage = function(canvasIndex, texId, imgId) {
  texCreate(canvasIndex, imageElementMap[imgId], texId)
}
