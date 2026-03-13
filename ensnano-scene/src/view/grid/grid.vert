#version 450

layout(location=0) in vec2 a_position;

layout(location=0) out flat uint v_grid_type;
layout(location=1) out vec2 v_tex_honey_coords;
layout(location=2) out vec2 v_tex_square_coords;
layout(location=3) out vec4 v_color;
layout(location=4) out flat uint v_fake;
layout(location=5) out flat uint v_design_id;


layout(set=0, binding=0)
uniform Uniforms {
    vec3 u_camera_position;
    mat4 u_view;
    mat4 u_proj;
};

layout(set=1, binding=0) buffer ModelBlock {
    readonly mat4 model_matrix2[];
};

struct Instances {
    mat4 model;
    float min_x;
    float max_x;
    float min_y;
    float max_y;
    vec4 color;
    uint grid_type;
    float helix_radius;
    float inter_helix_gap;
    uint design_id;
};

layout(set=2, binding=0) 
buffer InstancesBlock {
    readonly Instances instances[];
};



void main() {
    uint SQUARE_GRID_TYPE = 0;
    uint HONEYCOMB_GRID_TYPE = 1;
    uint ROTATED_HONEYCOMB_GRID_TYPE = 2;
    uint HYPERBOLOID_GRID_TYPE = 3;

    uint grid_type = instances[gl_InstanceIndex].grid_type;
    v_grid_type = grid_type % 1000;
    v_fake = grid_type / 1000;
    v_design_id = instances[gl_InstanceIndex].design_id;
    v_color = instances[gl_InstanceIndex].color;
    float u_helix_radius = instances[gl_InstanceIndex].helix_radius;
    float u_inter_helix_gap = instances[gl_InstanceIndex].inter_helix_gap;
    float r = u_helix_radius + u_inter_helix_gap / 2.;

    float min_x = instances[gl_InstanceIndex].min_x - 0.025;
    float max_x = instances[gl_InstanceIndex].max_x + 0.025;
    float min_y = -instances[gl_InstanceIndex].max_y - 0.025;
    float max_y = -instances[gl_InstanceIndex].min_y + 0.025;

    vec2 position = vec2((max_x - min_x) * a_position.x + min_x,
                    (max_y - min_y) * a_position.y + min_y);

    v_tex_honey_coords = position * vec2(1., -1.);
    v_tex_square_coords = position;

    mat4 design_matrix = model_matrix2[instances[gl_InstanceIndex].design_id];

    mat4 model_matrix = design_matrix * instances[gl_InstanceIndex].model;
    
    vec2 pos;
    if (v_grid_type == SQUARE_GRID_TYPE) {
        pos = 2 * r * position;
    } else if (v_grid_type == HONEYCOMB_GRID_TYPE) {
        pos = vec2(sqrt(3) * r,  3 * r) * position;
        pos.y += r;
    } else if (v_grid_type == ROTATED_HONEYCOMB_GRID_TYPE) {
        pos = vec2(-3 * r * position.y + r, sqrt(3) * r * position.x);
    } else if (v_grid_type == HYPERBOLOID_GRID_TYPE) {
        pos = position;
    }

    vec4 model_space = model_matrix * vec4(0., pos.y, pos.x, 1.0); 
    gl_Position = u_proj * u_view * model_space;
}
