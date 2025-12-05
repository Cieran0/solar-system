#version 330 core
in vec3 f_frag_pos;
in vec3 f_normal;
in vec2 tex_coord;
in vec4 f_frag_colour;

uniform sampler2D base_texture;
uniform bool use_texture;
uniform vec3 light_pos_world;

out vec4 frag_colour_out;

const vec3 global_ambient = vec3(0.25, 0.25, 0.25);

void main() {
    // Sample texture or use vertex color
    vec3 color = use_texture ? texture(base_texture, tex_coord).rgb : f_frag_colour.rgb;
    
    // Ambient lighting
    vec3 ambient = global_ambient * color;
    
    // Simple diffuse lighting
    vec3 norm = normalize(f_normal);
    vec3 lightDir = normalize(light_pos_world - f_frag_pos);
    float diff = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = diff * color;
    
    // Combine lighting
    vec3 result = (ambient + diffuse);
    
    // Final output with alpha
    frag_colour_out = vec4(result, 1.0);
}