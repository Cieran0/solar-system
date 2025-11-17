#version 420 core

in vec4 f_base_colour;
in vec3 f_position;
in vec3 f_light_direction;
in vec3 f_normal;
in vec2 f_tex_coord;

out vec4 outputColor;

uniform uint emit_mode;
uniform sampler2D base_texture;
uniform bool use_texture;

const vec3 global_ambient = vec3(0.25, 0.25, 0.25);
vec3 specular_albedo = vec3(1.0, 0.8, 0.6);
int shininess = 8;

void main()
{
    vec3 N = normalize(f_normal);
    vec3 L = normalize(f_light_direction);

    vec3 albedo = use_texture ? texture(base_texture, f_tex_coord).rgb : f_base_colour.rgb;

    vec3 ambient = albedo * global_ambient;
    float NdotL = max(dot(N, L), 0.0);
    vec3 diffuse = NdotL * albedo;

    vec3 V = normalize(-f_position);	
    vec3 R = reflect(-L, N);
    vec3 specular = pow(max(dot(R, V), 0.0), shininess) * specular_albedo;

    vec3 emissive = vec3(0.0);
    if (emit_mode == 1) {
        emissive = albedo * 2.0;
    }

    float dist = length(f_light_direction);
    float attenuation = 1.0 / (1.0 + 0.09 * dist + 0.032 * dist * dist);

    vec3 lit = (ambient + diffuse + specular) * attenuation;
    vec3 final = lit + emissive;

    float alpha = use_texture ? texture(base_texture, f_tex_coord).a : f_base_colour.a;
    outputColor = vec4(final, alpha);
}