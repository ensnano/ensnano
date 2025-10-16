#version 450

layout(location=0) in vec4 v_color;
layout(location=1) in vec3 v_normal;
layout(location=2) in vec3 v_position;
layout(location=3) in vec4 v_id;
flat layout(location=4) in uint v_discard_fake;

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
    float u_stereography_radius;
    mat4 u_stereography_view;
    float u_aspect_ratio;
    float u_stereography_zoom;
    uint u_nb_ray_tube;
    uint u_is_cut;
    vec3 u_cut_normal;
    float u_cut_dot_value;
};

const float HALF_LIFE = 10.;
const float GREY_VAL = pow(0.8, 2.2);
const vec3 BG_COLOR = vec3(GREY_VAL, GREY_VAL, GREY_VAL);

const vec3 SKY_COLOR = vec3(0.207, 0.321, 0.494);
const vec3 SAND_COLOR = vec3(0.368, 0.360, 0.219);
const vec3 HORIZON = vec3(0.917, 0.917, 0.917);
const vec3 DARK_FOG_COLOR = vec3(0.01, 0.01, 0.03);

void main() {
    f_color = vec4(v_color.xyz, 1.);
}
