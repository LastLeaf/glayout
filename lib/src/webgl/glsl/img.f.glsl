varying highp vec2 vTexPos;
varying highp float vTexIndex;
uniform sampler2D uTex0;
uniform sampler2D uTex1;
uniform sampler2D uTex2;
uniform sampler2D uTex3;
uniform sampler2D uTex4;
uniform sampler2D uTex5;
uniform sampler2D uTex6;
uniform sampler2D uTex7;
uniform sampler2D uTex8;
uniform sampler2D uTex9;
uniform sampler2D uTex10;
uniform sampler2D uTex11;
uniform sampler2D uTex12;
uniform sampler2D uTex13;
uniform sampler2D uTex14;
uniform sampler2D uTex15;

void main(void) {
  highp vec4 color = vec4(1, 0, 0, 0.1);
  if (vTexIndex == 0.) color = texture2D(uTex0, vTexPos);
  if (vTexIndex == 1.) color = texture2D(uTex1, vTexPos);
  if (vTexIndex == 2.) color = texture2D(uTex2, vTexPos);
  if (vTexIndex == 3.) color = texture2D(uTex3, vTexPos);
  if (vTexIndex == 4.) color = texture2D(uTex4, vTexPos);
  if (vTexIndex == 5.) color = texture2D(uTex5, vTexPos);
  if (vTexIndex == 6.) color = texture2D(uTex6, vTexPos);
  if (vTexIndex == 7.) color = texture2D(uTex7, vTexPos);
  if (vTexIndex == 8.) color = texture2D(uTex8, vTexPos);
  if (vTexIndex == 9.) color = texture2D(uTex9, vTexPos);
  if (vTexIndex == 10.) color = texture2D(uTex10, vTexPos);
  if (vTexIndex == 11.) color = texture2D(uTex11, vTexPos);
  if (vTexIndex == 12.) color = texture2D(uTex12, vTexPos);
  if (vTexIndex == 13.) color = texture2D(uTex13, vTexPos);
  if (vTexIndex == 14.) color = texture2D(uTex14, vTexPos);
  if (vTexIndex == 15.) color = texture2D(uTex15, vTexPos);
  gl_FragColor = color;
}
