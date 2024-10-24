// shader.frag
#version 450

layout(location=0) in vec4 v_color;
layout(location=1) in vec3 v_normal;
layout(location=2) in vec3 v_position;
layout(location=3) in vec4 v_id;
flat layout(location=4) in uint v_discard_fake; // NS DID NOT MANAGE TO MAKE IT WORK WITH THE PIPELINE

layout(location=0) out vec4 f_color;

layout(set=0, binding=0) uniform Uniform {
    uniform vec3 u_camera_position;
    mat4 u_view;
    mat4 u_proj;
    mat4 u_inversed_view;
    float u_fog_radius;
    float u_fog_length;
    uint u_make_fog;
    uint u_fog_from_cam;
    vec3 u_fog_center;
};

void main() {
    float visibility;

    if (v_discard_fake == 1) {
        discard;
    }

    if (u_make_fog > 0) {
        float dist;
        if (u_fog_from_cam > 0) {
           dist = length(u_camera_position - v_position);
        } else {
          dist = length(u_fog_center - v_position);
        }
        visibility =  1. - smoothstep(u_fog_length, u_fog_length + u_fog_radius, dist);
    } else {
        visibility = 1.;
    }

    if (u_make_fog == 3) {
       visibility = 1. - visibility;
    }

    if (visibility < 0.80 && (u_make_fog == 1 || u_make_fog == 3) ) {
     discard;
    }

    f_color = v_id;
}
