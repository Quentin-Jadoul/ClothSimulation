struct Position {
    position_x: f32,
    position_y: f32,
    position_z: f32,
    normal_x: f32,
    normal_y: f32,
    normal_z: f32,
    tangent_x: f32,
    tangent_y: f32,
    tangent_z: f32,
    tex_coords_x: f32,
    tex_coords_y: f32,
}

struct Velocity {
    velocity_x: f32,
    velocity_y: f32,
    velocity_z: f32,
}

struct ComputeData {
    delta_time: f32,
    nb_vertices: u32,
    sphere_radius: f32,
    sphere_center_x: f32,
    sphere_center_y: f32,
    sphere_center_z: f32,
    vertex_mass: f32,
    structural_stiffness: f32,
    shear_stiffness: f32,
    bend_stiffness: f32,
    structural_damping: f32,
    shear_damping: f32,
    bend_damping: f32,
}

struct Spring {
    vertex_index_1: f32,
    vertex_index_2: f32,
    rest_length: f32,
}

@group(0) @binding(0) var<storage, read_write> verticiesPositions: array<Position>;
@group(1) @binding(0) var<storage, read_write> verticiesVelocities: array<Velocity>;
@group(2) @binding(0) var<uniform> data: ComputeData;
@group(3) @binding(0) var<storage, read> springsR: array<Spring>;

@compute @workgroup_size(128, 1, 1)
fn main(@builtin(global_invocation_id) param: vec3<u32>) {
    if (param.x >= u32(data.nb_vertices)) {
          return;
    }
    
    var force_resultant = vec3<f32>(0.0, 0.0, 0.0);
    // for (var i = u32(4) * param.x ; i < u32(4) * param.x + u32(4); i = i + u32(1)) {
    //     let spring = springsR[i];
    //     let vertex_index_1 = spring.vertex_index_1;
    //     let vertex_index_2 = spring.vertex_index_2;
    //     let rest_length = spring.rest_length;

    //     if u32(spring.vertex_index_1) < u32(16) + u32(1){
    //         if u32(spring.vertex_index_2) < u32(16) + u32(1) {
    //             let position_1 = vec3<f32>(verticiesPositions[u32(vertex_index_1)].position_x, verticiesPositions[u32(vertex_index_1)].position_y, verticiesPositions[u32(vertex_index_1)].position_z);
    //             let position_2 = vec3<f32>(verticiesPositions[u32(vertex_index_2)].position_x, verticiesPositions[u32(vertex_index_2)].position_y, verticiesPositions[u32(vertex_index_2)].position_z);

    //             let current_length = distance(position_1,position_2);

    //             let force = -data.structural_stiffness * (current_length - springsR[i].rest_length);
    //             let direction = normalize(position_1 - position_2);
    //             //force_resultant += force * direction;
    //             // if u32(i) < u32(4) {
    //             // }
    //             // else if u32(i) < u32(8) {
    //             //     force_resultant.y += -9.81 * data.vertex_mass;
    //             //     let force = -data.shear_stiffness * (current_length - spring.rest_length);
    //             //     let direction = normalize(position_1 - position_2);
    //             //     //force_resultant += force * direction;
    //             // }
    //             // else if u32(i) < u32(12) {
    //             //     let force = -data.bend_stiffness * (current_length - spring.rest_length);
    //             //     let direction = normalize(position_1 - position_2);
    //             //     //force_resultant += force * direction;
    //             // }

    //         }
    //     }
    // }
    force_resultant.y += -9.81 * data.vertex_mass;

    // update the velocity of the vertex
    verticiesVelocities[param.x].velocity_x += (force_resultant.x / data.vertex_mass) * data.delta_time;
    verticiesVelocities[param.x].velocity_y += (force_resultant.y / data.vertex_mass) * data.delta_time;
    verticiesVelocities[param.x].velocity_z += (force_resultant.z / data.vertex_mass) * data.delta_time;

    // update the position of the vertex, or it crashes
    verticiesPositions[param.x].position_x += 0.0;
}