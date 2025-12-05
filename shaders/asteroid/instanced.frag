#version 330 core
in vec3 FragPos;
in vec3 Normal;
in vec2 TexCoord;
in vec4 FragColor;
in float Visibility;

uniform sampler2D base_texture;
uniform bool use_texture;
uniform vec3 light_pos_world;

out vec4 FragColorOut;

const vec3 global_ambient = vec3(0.25, 0.25, 0.25);

void main() {
    // Sample texture or use vertex color
    vec3 color = use_texture ? texture(base_texture, TexCoord).rgb : FragColor.rgb;
    
    // Ambient lighting
    vec3 ambient = global_ambient * color;
    
    // Simple diffuse lighting
    vec3 norm = normalize(Normal);
    vec3 lightDir = normalize(light_pos_world - FragPos);
    float diff = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = diff * color;
    
    // Combine lighting
    vec3 result = (ambient + diffuse) * Visibility;
    
    // Final output with alpha
    FragColorOut = vec4(result, 1.0);
}