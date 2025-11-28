#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec3 a_normal;

layout(location=0) out vec4 v_color;
layout(location=1) out vec3 v_normal;
layout(location=2) out vec3 v_position;
layout(location=3) out vec4 v_id;
flat layout(location=4) out uint v_discard_fake; 


layout(std140, set=0, binding=0)
uniform Uniforms {
    vec3 u_camera_position;
    mat4 u_view;
    mat4 u_proj;
    mat4 u_inversed_view;
    vec4 u_padding1;
    vec3 u_padding2;
    float u_stereography_radius;
    mat4 u_stereography_view;
    float u_aspect_ratio;
    float u_stereography_zoom;
    uint u_nb_ray_tube;
};

layout(set=1, binding=0) buffer ModelBlock {
    readonly mat4 model_matrix2[];
};

struct Instances {
    mat4 model;
    vec4 color;
    vec3 scale;
    uint id;
    mat4 inversed_model;
    vec3 prev;
    uint mesh;
    vec3 next;
};

layout(std430, set=2, binding=0) 
buffer InstancesBlock {
    readonly Instances instances[];
};

void main() {
    int model_idx = 0;

    mat4 model_matrix = model_matrix2[model_idx] * instances[gl_InstanceIndex].model;
    mat4 inversed_model_matrix = instances[gl_InstanceIndex].inversed_model;
    mat3 normal_matrix = mat3(transpose(inversed_model_matrix));

    // Note: I'm currently doing things in world space .
    // Doing things in view-space also known as eye-space, is more standard as objects can have
    // lighting issues when they are further away from the origin. 
    // If we wanted to use view-space, we would use something along the lines
    // of mat3(transpose(inverse(view_matrix * model_matrix))).
    // Currently we are combining the view matrix and projection matrix before we draw,
    // so we'd have to pass those in separately. We'd also have to transform our 
    // light's position using something like view_matrix * model_matrix

    v_normal = normal_matrix * a_normal;
    v_color = instances[gl_InstanceIndex].color;
    vec3 scale = instances[gl_InstanceIndex].scale;

    vec4 model_space = model_matrix * vec4(a_position * scale, 1.0); 

    if (instances[gl_InstanceIndex].mesh == 4 && v_color.w < 0.6) {
        v_discard_fake = 1;
    } else {
        v_discard_fake = 0;
    }

    v_position = model_space.xyz;
    uint id = instances[gl_InstanceIndex].id;
    v_id = vec4(
          float((id >> 16) & 0xFF) / 255.,
          float((id >> 8) & 0xFF) / 255.,
          float(id & 0xFF) / 255.,
          float((id >> 24) & 0xFF) / 255.);
    if (u_stereography_radius > 0.0) {
        vec4 view_space = u_stereography_view * model_space;
        view_space /= u_stereography_radius;
        float dist = length(view_space.xyz);
        vec3 projected = view_space.xyz / dist;
        float close_to_pole = 0.0;
        if (projected.z > (1. - (0.01 / u_stereography_zoom))) {
            close_to_pole = 1.0;
        }
        float z = max(close_to_pole, atan(dist) * 2. / 3.14);
        gl_Position = vec4(projected.x / (1. - projected.z) / u_stereography_zoom / u_aspect_ratio, projected.y / (1. - projected.z) / u_stereography_zoom, z, 1.);
    } else {
        gl_Position = u_proj * u_view * model_space;
    }
}
