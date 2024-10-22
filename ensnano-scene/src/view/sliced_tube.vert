#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec3 a_normal;

layout(location=0) out vec4 v_color;
layout(location=1) out vec3 v_normal;
layout(location=2) out vec3 v_position;
layout(location=3) out vec4 v_id;
layout(location=4) out uint v_discard_fake;

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
    mat4 model; // translation to position + rotation to align u_x to the axis of the tube
    vec4 color;
    vec3 scale;
    uint id;
    mat4 inversed_model;
    vec3 prev;
    uint mesh;
    vec3 next;
    float _padding;
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
    float epsilon = 1e-5;


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
    v_normal = a_normal;
    v_color = instances[gl_InstanceIndex].color;
    vec3 scale = instances[gl_InstanceIndex].scale;

    vec3 position = a_position * scale;
    vec3 normal = a_normal;

    if (instances[gl_InstanceIndex].mesh == 4 && v_color.w < 0.6) {
        v_discard_fake = 1;
    } else {
        v_discard_fake = 0;
    }


    // vec3 _next = vec3(1., 1., 1.);
    // vec3 _prev = vec3(0., 1., 0.);

    // in the referential of the tube
    vec3 prev = instances[gl_InstanceIndex].prev;
    vec3 next = instances[gl_InstanceIndex].next;

    float l_prev = length(prev);
    float l_next = length(next);
    vec3 vec_x = vec3(1., 0., 0.);

    if (l_prev > epsilon && abs(prev.y) + abs(prev.z) > epsilon && gl_VertexIndex < u_nb_ray_tube) {
        // left face -> compute intersection with prev
        prev /= l_prev; 
        // compute the normal to the intersection plane
        vec3 bisector = normalize(prev - vec_x); 
        vec3 perp_vec = cross(prev, vec_x);
        vec3 plane_normal = normalize(cross(bisector, perp_vec));
        // project the point on the intersection plane
        position.x -= (plane_normal.y * position.y + plane_normal.z * position.z) / plane_normal.x; 
        // compute the normal by projecting the tangent on the intersection plane and taking the cross product to get a normal in the plane and perpendicular to the tangent
        vec3 tangent = vec3(0., a_normal.z, -a_normal.y);
        tangent.x = -(plane_normal.y * tangent.y + plane_normal.z * tangent.z) / plane_normal.x;
        v_normal = -normalize(cross(plane_normal,tangent));
    } else if (l_next > epsilon && abs(next.y) + abs(next.z) > epsilon  && gl_VertexIndex >= 2 * u_nb_ray_tube) {
        // right face -> compute intersection with next
        next /= l_next;
        vec3 bisector = normalize(vec_x - next); 
        vec3 perp_vec = cross(vec_x, next);
        vec3 plane_normal = normalize(cross(bisector, perp_vec));
        position.x -= (plane_normal.y * position.y + plane_normal.z * position.z) / plane_normal.x; 
        vec3 tangent = vec3(0., -a_normal.z, a_normal.y);
        tangent.x = -(plane_normal.y * tangent.y + plane_normal.z * tangent.z) / plane_normal.x;
        v_normal = normalize(cross(plane_normal,tangent));
    }
    // } else {
    //     // middle face - rien à changer
    // }


    vec4 model_space = model_matrix * vec4(position, 1.0); 
    v_position = model_space.xyz;
    v_normal = normal_matrix * v_normal;

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
