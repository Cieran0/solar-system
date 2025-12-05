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
out vec3 Normal;
out vec2 TexCoord;
out vec4 FragColor;
out float Visibility;

const float PI = 3.14159265359;
uniform float shadow_far;

float calculateVisibility(float distance, float far) {
    float normalizedDistance = distance / far;
    return 1.0 - (normalizedDistance * normalizedDistance);
}

void main() {
    vec4 worldPos = instanceModel * vec4(aPos, 1.0);
    f_frag_pos = worldPos.xyz;
    Normal = normalize(normal_matrix * aNormal);
    TexCoord = aTexCoord;
    FragColor = aColor;
    
    float distance = length(worldPos.xyz - light_pos_world);
    Visibility = clamp(calculateVisibility(distance, shadow_far), 0.0, 1.0);
    
    gl_Position = projection * view * worldPos;
}