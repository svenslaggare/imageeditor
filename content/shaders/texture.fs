#version 330 core
in vec2 texCoord;

uniform sampler2D inputTexture;

out vec4 outputColor;

void main() {
    outputColor = texture(inputTexture, texCoord).rgba;
}