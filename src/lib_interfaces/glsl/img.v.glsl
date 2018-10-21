#ifdef GL_ES
#else
#define highp
#define mediump
#define lowp
#endif

attribute vec2 aDrawPos;
attribute vec2 aTexPos;
attribute float aTexIndex;
varying highp vec2 vTexPos;
varying highp float vTexIndex;
uniform vec3 uAreaSize;

void main(void) {
  gl_Position = vec4(aDrawPos * mat2(2.0/uAreaSize.x,0, 0,-2.0/uAreaSize.y*uAreaSize.z) + vec2(-1, uAreaSize.z), 0, 1);
  vTexPos = aTexPos;
  vTexIndex = aTexIndex;
}
