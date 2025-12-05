#version 420 core
in vec4 f_base_colour;
in vec3 f_position_world;
in vec3 f_view_position;
in vec3 f_light_direction;
in vec3 f_normal;
in vec2 f_tex_coord;
in vec3 FragPos;
out vec4 outputColor;

uniform uint emit_mode;
uniform sampler2D base_texture;
uniform bool use_texture;
uniform samplerCubeShadow shadow_map;
uniform float shadow_far;
uniform vec3 light_pos_world;

const vec3 global_ambient = vec3(0.25, 0.25, 0.25);
vec3 specular_albedo = vec3(1.0, 0.8, 0.6);
int shininess = 8;

float calculate_shadow() {
    vec3 light_to_frag = FragPos - light_pos_world;
    float dist = length(light_to_frag);
    if (dist > shadow_far)
        return 0.0;
    vec3 dir = normalize(light_to_frag);
    float depth = dist / shadow_far;
    float bias = 0.002;
    float shadow = 0.0;
    float offset = 0.002;
    int samples = 0;
    for(int x = -1; x <= 1; x++)
    for(int y = -1; y <= 1; y++)
    for(int z = -1; z <= 1; z++) {
        vec3 sampleDir = dir + vec3(x, y, z) * offset;
        shadow += 1.0 - texture(shadow_map, vec4(sampleDir, depth - bias));
        samples++;
    }
    return shadow / float(samples);
}

void main() {
    vec3 N = normalize(f_normal);
    vec3 L = normalize(f_light_direction);
    vec3 albedo = use_texture ? texture(base_texture, f_tex_coord).rgb : f_base_colour.rgb;
    vec3 ambient = albedo * global_ambient;
    float NdotL = max(dot(N, L), 0.0);
    vec3 diffuse = NdotL * albedo;

    vec3 V = normalize(-f_view_position);

    vec3 R = reflect(-L, N);
    //vec3 specular = pow(max(dot(R, V), 0.0), shininess) * specular_albedo;

    vec3 emissive = (emit_mode == 1) ? albedo * 2.0 : vec3(0.0);

    float dist = length(f_light_direction);
    float attenuation = 1.0 / (1.0 + 0.09 * dist + 0.032 * dist * dist);

    float shadow = calculate_shadow();
    //vec3 lit = (ambient + (1.0 - shadow) * (diffuse + specular)) * attenuation;
    vec3 lit = (ambient + (1.0 - shadow) * (diffuse)) * attenuation;
    vec3 final = lit + emissive;

    float alpha = use_texture ? texture(base_texture, f_tex_coord).a : f_base_colour.a;
    outputColor = vec4(final, alpha);
}