import {ELEMENT_TYPE_MAP, STR_BUF_LEN} from './index'

export class Element {
  static _create(context, name) {
    let [ElemConstructor, typeId] = ELEMENT_TYPE_MAP[name]
    let ret = new ElemConstructor()
    ret._ptr = __glayoutAsm__._element_new(context, typeId)
    return ret
  }
  static _from_ptr(ptr) {
    if (ptr === 0) return null
    let ret = new Element()
    ret._ptr = ptr
    return ret
  }
  release() {
    __glayoutAsm__._release_node(this._ptr)
  }

  getParentNode() {
    return Element._from_ptr(__glayoutAsm__._element_parent(this._ptr))
  }
  getChildNode(index) {
    return Element._from_ptr(__glayoutAsm__._element_child(this._ptr, index))
  }
  appendChild(child) {
    Element._from_ptr(__glayoutAsm__._element_append(this._ptr, child._ptr))
  }
  insertChild(child, index) {
    Element._from_ptr(__glayoutAsm__._element_insert(this._ptr, child._ptr, index))
  }
  removeChild(index) {
    Element._from_ptr(__glayoutAsm__._element_remove(this._ptr, index))
  }
  node_under_point(x, y) {
    return Element._from_ptr(__glayoutAsm__._element_node_under_point(this._ptr, x, y))
  }

  setClass(str) {
    const bufAddr = __glayoutAsm__._get_swap_buffer(STR_BUF_LEN)
    __glayoutAsm__.stringToUTF8(str, bufAddr, STR_BUF_LEN)
    __glayoutAsm__._element_class(this._ptr, bufAddr)
  }
  setStyle(str) {
    const bufAddr = __glayoutAsm__._get_swap_buffer(STR_BUF_LEN)
    __glayoutAsm__.stringToUTF8(str, bufAddr, STR_BUF_LEN)
    __glayoutAsm__._element_style(this._ptr, bufAddr)
  }
}
