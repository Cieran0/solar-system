#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec4 aColor;
layout (location = 2) in vec3 aNormal;
layout (location = 3) in vec2 aTexCoord;
layout (location = 4) in mat4 instanceModel;

uniform mat4 view;
uniform mat4 projection;
uniform mat3 normal_matrix;
uniform vec3 light_pos_world;

out vec3 f_frag_pos;
out vec3 f_normal;
out vec2 tex_coord;
out vec4 f_frag_colour;

uniform float shadow_far;

void main() {
    vec4 worldPos = instanceModel * vec4(aPos, 1.0);
    f_frag_pos = worldPos.xyz;
    f_normal = normalize(normal_matrix * aNormal);
    tex_coord = aTexCoord;
    f_frag_colour = aColor;
    
    float distance = length(worldPos.xyz - light_pos_world);
    
    gl_Position = projection * view * worldPos;
}