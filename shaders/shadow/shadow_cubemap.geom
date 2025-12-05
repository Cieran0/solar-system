#version 420 core
layout(triangles) in;
layout(triangle_strip, max_vertices = 18) out;

uniform mat4 light_matrix[6]; // projection * view for each cube face
in vec3 frag_pos[]; // from vertex shader
out vec3 FragPos;   // to fragment shader

void main() {
    // Emit each triangle three times (once per cube face)
    for (int face = 0; face < 6; ++face) {
        for (int i = 0; i < 3; ++i) {
            FragPos = frag_pos[i];
            gl_Layer = face; // layered rendering to cubemap
            gl_Position = light_matrix[face] * vec4(FragPos, 1.0);
            EmitVertex();
        }
        EndPrimitive();
    }
}
