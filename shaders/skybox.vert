#version 420 core
layout(location = 0) in vec3 position;
uniform mat4 projection;
uniform mat4 view;

out vec3 TexCoords;

void main()
{
    TexCoords = position;
    vec4 pos = projection * view * vec4(position, 1.0);
    gl_Position = pos.xyww; // Force depth to 1.0 (far plane)
}