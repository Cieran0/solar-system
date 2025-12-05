#version 420 core
layout(location = 0) in vec3 position;
layout(location = 1) in vec4 colour;
layout(location = 2) in vec3 normal;
layout(location = 3) in vec2 tex_coord;
out vec4 f_base_colour;
out vec3 f_position_world;
out vec3 f_view_position;
out vec3 f_light_direction;
out vec3 f_normal;
out vec2 f_tex_coord;
out vec3 FragPos;
uniform mat4 model, view, projection;
uniform mat3 normal_matrix;
uniform vec4 light_pos;
void main() {
    vec4 world_pos = model * vec4(position, 1.0);
    vec4 view_pos = view * world_pos;

    vec4 diffuse_albedo = colour;
    vec3 N = normalize(normal_matrix * normal);
    vec3 light_pos3 = light_pos.xyz;
    vec3 L = light_pos3 - view_pos.xyz;

    f_base_colour = diffuse_albedo;
    f_position_world = world_pos.xyz;
    f_view_position = view_pos.xyz;
    f_light_direction = L;
    f_normal = N;
    f_tex_coord = tex_coord;
    FragPos = world_pos.xyz;

    gl_Position = projection * view_pos;
}