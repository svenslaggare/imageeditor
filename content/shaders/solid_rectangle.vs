#version 330 core
layout (location = 0) in vec2 vertexPosition;
layout (location = 1) in vec3 vertexColor;

uniform mat4 transform;

out vec3 color;

void main() {
	gl_Position = transform * vec4(vertexPosition, 0.0, 1.0);
	color = vertexColor;
}