#version 330 core
layout (location = 0) in vec2 vertexPosition;
layout (location = 1) in vec2 vertexTexCoord;

uniform mat4 transform;

out vec2 texCoord;

void main() {
	gl_Position = transform * vec4(vertexPosition, 0.0, 1.0);
	texCoord = vertexTexCoord;
}