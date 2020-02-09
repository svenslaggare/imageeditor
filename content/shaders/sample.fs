#version 330 core
in vec2 texCoord;

uniform sampler2D inputTexture;

out vec4 outputColor;

void main() {
    outputColor = texture(inputTexture, texCoord).rgba + vec4(0.15, 0.0, 0.0, 0.0);

//     float blurX = 0.0;
//     float blurY = 0.002;
//
//     float hstep = 0.0;
//     float vstep = 1.0;
//
//    outputColor += texture(inputTexture, vec2(texCoord.x - 4.0 * blurX * hstep, texCoord.y - 4.0 * blurY * vstep)) * 0.0162162162;
//    outputColor += texture(inputTexture, vec2(texCoord.x - 3.0 * blurX * hstep, texCoord.y - 3.0 * blurY * vstep)) * 0.0540540541;
//    outputColor += texture(inputTexture, vec2(texCoord.x - 2.0 * blurX * hstep, texCoord.y - 2.0 * blurY * vstep)) * 0.1216216216;
//    outputColor += texture(inputTexture, vec2(texCoord.x - 1.0 * blurX * hstep, texCoord.y - 1.0 * blurY * vstep)) * 0.1945945946;
//
//    outputColor += texture(inputTexture, vec2(texCoord.x, texCoord.y)) * 0.2270270270;
//
//    outputColor += texture(inputTexture, vec2(texCoord.x + 1.0 * blurX * hstep, texCoord.y + 1.0 * blurY * vstep)) * 0.1945945946;
//    outputColor += texture(inputTexture, vec2(texCoord.x + 2.0 * blurX * hstep, texCoord.y + 2.0 * blurY * vstep)) * 0.1216216216;
//    outputColor += texture(inputTexture, vec2(texCoord.x + 3.0 * blurX * hstep, texCoord.y + 3.0 * blurY * vstep)) * 0.0540540541;
//    outputColor += texture(inputTexture, vec2(texCoord.x + 4.0 * blurX * hstep, texCoord.y + 4.0 * blurY * vstep)) * 0.0162162162;
}