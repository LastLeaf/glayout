import { canvases } from './canvas_store'

export const imageLoadUrl = function(canvasIndex, id, url, cbPtr) {
  const {imageElementMap} = canvases[canvasIndex]
  const imgElem = document.createElement('img')
  imgElem.onload = function() {
    imageElementMap[id] = imgElem
    __glayoutAsm__._callback(cbPtr, 0)
  }
  imgElem.onerror = imgElem.onabort = function(){
    __glayoutAsm__._callback(cbPtr, -1)
  }
  imgElem.src = __glayoutAsm__.UTF8ToString(url)
}

export const imageUnload = function(canvasIndex, id) {
  const {imageElementMap} = canvases[canvasIndex]
  delete imageElementMap[id]
}
