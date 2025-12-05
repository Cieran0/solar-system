#version 420 core

layout(location = 0) in vec3 position;

uniform mat4 model;

out vec3 frag_pos; // world-space position

void main() {
    vec4 world_pos = model * vec4(position, 1.0);
    frag_pos = world_pos.xyz;
}
