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
}

@group(0) @binding(0) var<storage, read_write> verticiesPositions: array<Position>;
@group(1) @binding(0) var<storage, read_write> verticiesVelocities: array<Velocity>;
@group(2) @binding(0) var<uniform> data: ComputeData;

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) param: vec3<u32>) {
    if (param.x >= u32(data.nb_vertices)) {
          return;
    }

    //var position = verticiesPositions[param.x];
    //var velocity = verticiesVelocities[param.x];

    verticiesPositions[param.x].position_x += verticiesVelocities[param.x].velocity_x * data.delta_time;
    verticiesPositions[param.x].position_y += verticiesVelocities[param.x].velocity_y * data.delta_time;
    verticiesPositions[param.x].position_z += verticiesVelocities[param.x].velocity_z * data.delta_time;

    verticiesVelocities[param.x].velocity_y += -9.81 * data.delta_time;
}