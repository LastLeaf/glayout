import {Element} from './element'
import {STR_BUF_LEN} from './index'

export class Canvas {
  constructor(index) {
    this._ptr = __glayoutAsm__._canvas_create(index)
  }
  destroy() {
    __glayoutAsm__._canvas_create(this._ptr)
  }
  getContext() {
    return new CanvasContext(this)
  }
}

export class CanvasContext {
  constructor(canvas) {
    this._ptr = __glayoutAsm__._canvas_get_context(canvas._ptr)
  }
  // eslint-disable-next-line class-methods-use-this
  getWindowWidth() {
    return document.documentElement.clientWidth
  }
  // eslint-disable-next-line class-methods-use-this
  getWindowHeight() {
    return document.documentElement.clientHeight
  }
  // eslint-disable-next-line class-methods-use-this
  getDevicePixelRatio() {
    return window.devicePixelRatio
  }
  setCanvasSize(w, h, pixelRatio) {
    __glayoutAsm__._canvas_context_set_canvas_size(this._ptr, w, h, pixelRatio)
  }
  setClearColor(r, g, b, a) {
    __glayoutAsm__._canvas_context_set_clear_color(this._ptr, r, g, b, a)
  }
  // eslint-disable-next-line class-methods-use-this
  render(cb) {
    cb()
  }
  appendStyleSheet(styleText) {
    const bufAddr = __glayoutAsm__._get_swap_buffer(STR_BUF_LEN)
    __glayoutAsm__.stringToUTF8(styleText, bufAddr, STR_BUF_LEN)
    return __glayoutAsm__._canvas_context_append_style_sheet(this._ptr, bufAddr)
  }
  replaceStyleSheet(index, styleText) {
    const bufAddr = __glayoutAsm__._get_swap_buffer(STR_BUF_LEN)
    __glayoutAsm__.stringToUTF8(styleText, bufAddr, STR_BUF_LEN)
    __glayoutAsm__._canvas_context_replace_style_sheet(this._ptr, index, bufAddr)
  }
  clearStyleSheets() {
    __glayoutAsm__._canvas_context_clear_style_sheets(this._ptr)
  }
  getRootNode() {
    return Element._from_ptr(__glayoutAsm__._canvas_context_root(this._ptr))
  }
  createElement(name, tagName) {
    return Element._create(this._ptr, name, tagName)
  }
}
