#version 420 core

in vec3 f_frag_pos;
uniform vec3 light_pos;
uniform float shadow_far;

void main() {
    // distance from light to fragment
    float dist = length(f_frag_pos - light_pos);

    // normalized distance in [0,1]
    float nd = dist / shadow_far;

    gl_FragDepth = nd;
}
