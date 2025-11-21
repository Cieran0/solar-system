#version 420 core

// Fragment shader: write normalized distance into depth buffer (so samplerCubeShadow can compare)
in vec3 FragPos;
uniform vec3 light_pos;
uniform float shadow_far;

void main() {
    // distance from light to fragment
    float dist = length(FragPos - light_pos);

    // normalized distance in [0,1]
    float nd = dist / shadow_far;

    // Write normalized linear depth into the depth buffer.
    // This means when sampling with samplerCubeShadow, pass reference = dist / shadow_far.
    // No extra bias here (bias should be applied during comparison or in lighting shader).
    gl_FragDepth = nd;
}
