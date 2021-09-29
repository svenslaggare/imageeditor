#version 330 core
in vec3 color;
in vec2 texCoord;

uniform sampler2D inputTexture;

out vec4 outputColor;

void main() {
	vec4 sampled = vec4(1.0, 1.0, 1.0, texture(inputTexture, texCoord).r);
    outputColor = vec4(color, 1.0) * sampled;
}