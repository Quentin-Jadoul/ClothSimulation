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
}

@group(0) @binding(0) var<storage, read_write> verticiesPositions: array<Position>;
@group(1) @binding(0) var<storage, read_write> verticiesVelocities: array<Velocity>;
@group(2) @binding(0) var<uniform> data: ComputeData;

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) param: vec3<u32>) {
    if (param.x >= u32(data.nb_vertices)) {
          return;
    }

    verticiesPositions[param.x].position_x += verticiesVelocities[param.x].velocity_x * data.delta_time;
    verticiesPositions[param.x].position_y += verticiesVelocities[param.x].velocity_y * data.delta_time;
    verticiesPositions[param.x].position_z += verticiesVelocities[param.x].velocity_z * data.delta_time;

    verticiesVelocities[param.x].velocity_y += -9.81 * data.delta_time;

    // some collision detection with the sphere
    // the sphere is centered at (0, 0, 0) and has a radius of 10
    // calculate the distance between the vertex and the extremity of the sphere

    let sphere_center = vec3<f32>(data.sphere_center_x, data.sphere_center_y, data.sphere_center_z);
    //let sphere_center = vec3<f32>(0.0, 0.0, 0.0);
    let sphere_radius = data.sphere_radius;
    let position = vec3<f32>(verticiesPositions[param.x].position_x, verticiesPositions[param.x].position_y, verticiesPositions[param.x].position_z);

    let distance = length(position - sphere_center);

    if (distance < sphere_radius) {
        // the vertex is inside the sphere
        // we need to push it out of the sphere
        // we do this by calculating the normal of the sphere at the vertex position
        // and then we push the vertex in the opposite direction of the normal

        let normal = normalize(position - sphere_center);

        verticiesPositions[param.x].position_x += normal.x * (sphere_radius - distance);
        verticiesPositions[param.x].position_y += normal.y * (sphere_radius - distance);
        verticiesPositions[param.x].position_z += normal.z * (sphere_radius - distance);

        verticiesVelocities[param.x].velocity_x = 0.0;
        verticiesVelocities[param.x].velocity_y = 0.0;
        verticiesVelocities[param.x].velocity_z = 0.0;
    }
    


}