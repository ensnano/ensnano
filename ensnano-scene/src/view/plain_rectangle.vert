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

/*
// expected_length is 0.64 nm for DNA if normal view and 2.65 if axis view
const float LOW_CRIT = 1. / 0.7; // bond starts getting grey if length > expected_length / 0.7, i.e. if 42% too high
const float HIGH_CRIT = 2. / 0.7; // bond gets black if length > 2*expected_length /0.7, i.e. if 185% too high
*/

void main() {
    int model_idx = 0;

    //mat4 model_matrix = model_matrix2[model_idx] * instances[gl_InstanceIndex].model;
    mat4 model_matrix = model_matrix2[model_idx] * instances[gl_InstanceIndex].model;
    mat4 inversed_model_matrix = instances[gl_InstanceIndex].inversed_model;
    mat3 normal_matrix = mat3(transpose(inversed_model_matrix));

    /*Note: I'm currently doing things in world space .
    Doing things in view-space also known as eye-space, is more standard as objects can have
    lighting issues when they are further away from the origin. 
    If we wanted to use view-space, we would use something along the lines
    of mat3(transpose(inverse(view_matrix * model_matrix))).
    Currently we are combining the view matrix and projection matrix before we draw,
    so we'd have to pass those in separately. We'd also have to transform our 
    light's position using something like view_matrix * model_matrix * */
    v_normal = vec3(0,0,-1);
    v_color = instances[gl_InstanceIndex].color;
    vec3 scale = instances[gl_InstanceIndex].scale;

    vec4 model_space = model_matrix * vec4(a_position * scale, 1.0); 

	// y is in -0.5..0.5
	// float a = - (0.5 + model_space.y) * 2. * 3.141592653589793;
	// float r = 0.15*(2.*model_space.x + 4.*model_space.y);
	// model_space =  vec4(r * cos(a), r * sin(a), 0., 1.);

    v_discard_fake = 1;
    
    v_position = model_space.xyz;
	gl_Position = vec4(v_position.xyz, 1.);
    uint id = instances[gl_InstanceIndex].id;
    v_id = vec4(
          float((id >> 16) & 0xFF) / 255.,
          float((id >> 8) & 0xFF) / 255.,
          float(id & 0xFF) / 255.,
          float((id >> 24) & 0xFF) / 255.);
}
