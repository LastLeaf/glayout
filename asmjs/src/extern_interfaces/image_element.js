import {Element} from './element'

const URL_BUF_LEN = 1024 * 1024

export class ImageElement extends Element {
  load(str) {
    const bufAddr = __glayoutAsm__._get_swap_buffer(URL_BUF_LEN)
    __glayoutAsm__.stringToUTF8(str, bufAddr, URL_BUF_LEN)
    __glayoutAsm__._image_element_load(this._ptr, bufAddr)
  }
}
