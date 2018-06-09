attribute vec2 aDrawPos;
attribute vec2 aTexPos;
attribute float aTexIndex;
varying highp vec2 vTexPos;
varying highp float vTexIndex;
uniform vec2 uAreaSize;

void main(void) {
  gl_Position = vec4(aDrawPos * mat2(2.0/uAreaSize.x,0, 0,-2.0/uAreaSize.y) + vec2(-1, 1), 0, 1);
  vTexPos = vec2(aTexPos * mat2(1.0/uAreaSize.x,0, 0,1.0/uAreaSize.y));
  vTexIndex = aTexIndex;
}
