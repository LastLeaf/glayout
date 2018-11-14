import {Element} from './element'
import {STR_BUF_LEN} from './index'

export class ImageElement extends Element {
  load(str) {
    const bufAddr = __glayoutAsm__._get_swap_buffer(STR_BUF_LEN)
    __glayoutAsm__.stringToUTF8(str, bufAddr, STR_BUF_LEN)
    __glayoutAsm__._image_element_load(this._ptr, bufAddr)
  }
}
