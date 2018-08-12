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
uniform mediump vec4 uColor;
uniform mediump float uAlpha;

void main(void) {
  mediump vec4 color;
  highp int texIndex = int(vTexIndex);
  bool useColor = true;
  if (vTexIndex < -1.5) {
    // draw rect instead of texture
    gl_FragColor = uColor * uAlpha;
  } else {
    if (texIndex >= 256) {
      texIndex -= 256;
      useColor = false;
    }
    if (texIndex == 0) color = texture2D(uTex0, vTexPos);
    if (texIndex == 1) color = texture2D(uTex1, vTexPos);
    if (texIndex == 2) color = texture2D(uTex2, vTexPos);
    if (texIndex == 3) color = texture2D(uTex3, vTexPos);
    if (texIndex == 4) color = texture2D(uTex4, vTexPos);
    if (texIndex == 5) color = texture2D(uTex5, vTexPos);
    if (texIndex == 6) color = texture2D(uTex6, vTexPos);
    if (texIndex == 7) color = texture2D(uTex7, vTexPos);
    if (texIndex == 8) color = texture2D(uTex8, vTexPos);
    if (texIndex == 9) color = texture2D(uTex9, vTexPos);
    if (texIndex == 10) color = texture2D(uTex10, vTexPos);
    if (texIndex == 11) color = texture2D(uTex11, vTexPos);
    if (texIndex == 12) color = texture2D(uTex12, vTexPos);
    if (texIndex == 13) color = texture2D(uTex13, vTexPos);
    if (texIndex == 14) color = texture2D(uTex14, vTexPos);
    if (texIndex == 15) color = texture2D(uTex15, vTexPos);
    if (useColor) {
      gl_FragColor = uColor * color.a * uAlpha;
    } else {
      gl_FragColor = color * uAlpha;
    }
  }
}
