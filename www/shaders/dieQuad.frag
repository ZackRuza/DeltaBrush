uniform sampler2D tDiffuse;
uniform float opacity;
varying vec2 vUv;

void main() {
    vec4 texColor = texture2D(tDiffuse, vUv);
    if (texColor.a < 0.01) discard;
    gl_FragColor = vec4(texColor.rgb, texColor.a * opacity);
}
