import {EmptyElement} from './empty_element'
import {TextElement} from './text_element'
import {ImageElement} from './image_element'

export * from './canvas'
export * from './element'

export const STR_BUF_LEN = 1024 * 1024

export const ELEMENT_TYPE_MAP = {
  "empty": [EmptyElement, 0],
  "text": [TextElement, 1],
  "fragment": [EmptyElement, 0],
  "image": [ImageElement, 2],
}
