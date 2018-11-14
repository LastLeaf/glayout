import {Element} from './element'
import {STR_BUF_LEN} from './index'

export class TextElement extends Element {
  setText(str) {
    const bufAddr = __glayoutAsm__._get_swap_buffer(STR_BUF_LEN)
    __glayoutAsm__.stringToUTF8(str, bufAddr, STR_BUF_LEN)
    __glayoutAsm__._text_element_set_text(this._ptr, bufAddr)
  }
}
