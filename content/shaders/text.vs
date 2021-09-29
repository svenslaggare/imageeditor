#version 330 core
layout (location = 0) in vec2 vertexPos;
layout (location = 1) in vec2 vertexTexCoord;
layout (location = 2) in vec3 vertexColor;

uniform mat4 transform;

out vec3 color;
out vec2 texCoord;

void main() {
	gl_Position = transform * vec4(vertexPos, 0.0, 1.0);
	color = vertexColor;
	texCoord = vertexTexCoord;
}