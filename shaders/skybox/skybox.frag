#version 420 core
in vec3 texture_coords;
out vec4 frag_colour;

uniform samplerCube skybox;

void main()
{    
    frag_colour = texture(skybox, normalize(texture_coords));
}