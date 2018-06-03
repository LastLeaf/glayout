varying highp vec2 vTexPos;
varying highp float vTexIndex;
uniform sampler2D vTex0;
uniform sampler2D vTex1;
uniform sampler2D vTex2;
uniform sampler2D vTex3;
uniform sampler2D vTex4;
uniform sampler2D vTex5;
uniform sampler2D vTex6;
uniform sampler2D vTex7;
uniform sampler2D vTex8;
uniform sampler2D vTex9;
uniform sampler2D vTex10;
uniform sampler2D vTex11;
uniform sampler2D vTex12;
uniform sampler2D vTex13;
uniform sampler2D vTex14;
uniform sampler2D vTex15;

void main(void) {
  highp vec4 color = vec4(0, 0, 0, 0);
  if (vTexIndex == 0.) color = texture2D(vTex0, vec2(vTexPos.s, vTexPos.t));
  if (vTexIndex == 1.) color = texture2D(vTex1, vec2(vTexPos.s, vTexPos.t));
  if (vTexIndex == 2.) color = texture2D(vTex2, vec2(vTexPos.s, vTexPos.t));
  if (vTexIndex == 3.) color = texture2D(vTex3, vec2(vTexPos.s, vTexPos.t));
  if (vTexIndex == 4.) color = texture2D(vTex4, vec2(vTexPos.s, vTexPos.t));
  if (vTexIndex == 5.) color = texture2D(vTex5, vec2(vTexPos.s, vTexPos.t));
  if (vTexIndex == 6.) color = texture2D(vTex6, vec2(vTexPos.s, vTexPos.t));
  if (vTexIndex == 7.) color = texture2D(vTex7, vec2(vTexPos.s, vTexPos.t));
  if (vTexIndex == 8.) color = texture2D(vTex8, vec2(vTexPos.s, vTexPos.t));
  if (vTexIndex == 9.) color = texture2D(vTex9, vec2(vTexPos.s, vTexPos.t));
  if (vTexIndex == 10.) color = texture2D(vTex10, vec2(vTexPos.s, vTexPos.t));
  if (vTexIndex == 11.) color = texture2D(vTex11, vec2(vTexPos.s, vTexPos.t));
  if (vTexIndex == 12.) color = texture2D(vTex12, vec2(vTexPos.s, vTexPos.t));
  if (vTexIndex == 13.) color = texture2D(vTex13, vec2(vTexPos.s, vTexPos.t));
  if (vTexIndex == 14.) color = texture2D(vTex14, vec2(vTexPos.s, vTexPos.t));
  if (vTexIndex == 15.) color = texture2D(vTex15, vec2(vTexPos.s, vTexPos.t));
  gl_FragColor = color;
}
