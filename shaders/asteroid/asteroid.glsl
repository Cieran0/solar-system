#version 430
layout(local_size_x = 256) in;

struct AsteroidStatic {
    float orbit_radius;
    float inclination;
    float scale;
    float y_axis_offset;
};
layout(std430, binding = 0) buffer AsteroidStaticData { AsteroidStatic asteroid_static[]; };

struct AsteroidDynamic {
    float orbit_angle;
    float rotation;
    float orbit_speed;
    float rotation_speed;
};
layout(std430, binding = 1) buffer AsteroidDynamicData { AsteroidDynamic asteroid_dynamic[]; };

layout(std430, binding = 2) buffer MatrixData { mat4 model[]; };

uniform float dt;

void main() {
    uint id = gl_GlobalInvocationID.x;
    if (id >= asteroid_dynamic.length()) return;

    AsteroidStatic s = asteroid_static[id];
    AsteroidDynamic d = asteroid_dynamic[id];

    // Update orbit and rotation
    d.orbit_angle += d.orbit_speed * dt;
    d.rotation += d.rotation_speed * dt;

    // Inline math for orbit and rotations
    float cos_a = cos(d.orbit_angle);
    float sin_a = sin(d.orbit_angle);
    vec3 pos;
    pos.x = s.orbit_radius * cos_a;
    pos.y = 0.0;
    pos.z = s.orbit_radius * sin_a;

    // Tilt around X axis (inclination)
    float cosI = cos(s.inclination);
    float sinI = sin(s.inclination);
    float y = pos.y * cosI - pos.z * sinI;
    float z = pos.y * sinI + pos.z * cosI;
    pos.y = y;
    pos.z = z;

    // Random Y-axis spread
    float cosY = cos(s.y_axis_offset);
    float sinY = sin(s.y_axis_offset);
    float x = pos.x * cosY + pos.z * sinY;
    z = -pos.x * sinY + pos.z * cosY;
    pos.x = x;
    pos.z = z;

    // Build model matrix directly
    float c = cos(d.rotation);
    float s_rot = sin(d.rotation);
    mat4 m = mat4(1.0);

    m[0][0] = s.scale * c; m[0][2] = s.scale * s_rot;
    m[1][1] = s.scale;
    m[2][0] = s.scale * -s_rot; m[2][2] = s.scale * c;
    m[3] = vec4(pos, 1.0);

    model[id] = m;
    asteroid_dynamic[id] = d;
}
