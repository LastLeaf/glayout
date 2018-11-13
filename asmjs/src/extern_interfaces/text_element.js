import {Element} from './element'

const TEXT_BUF_LEN = 1024 * 1024

export class TextElement extends Element {
  setText(str) {
    const bufAddr = __glayoutAsm__._get_swap_buffer(TEXT_BUF_LEN)
    __glayoutAsm__.stringToUTF8(str, bufAddr, TEXT_BUF_LEN)
    __glayoutAsm__._text_element_set_text(this._ptr, bufAddr)
  }
}
