// display a sphere in the middle of the screen with a compute shader

use wgpu_bootstrap::{
    window::Window,
    frame::Frame,
    application::Application,
    context::Context,
    geometry::icosphere,
    camera::Camera,
    wgpu,
    cgmath,
    default::Vertex,
    computation::Computation,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ComputeData {
    delta_time: f32,
    nb_vertices: u32,
    // sphere_radius: u32,
    // sphere_center: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Velocity {
    pub velocity: [f32; 3]
}

// we want to change the size of the cloth, the number of vertices and the start position
const CLOTH_SIZE: u32 = 25;
const N_CLOTH_VERTICES_PER_ROW: u32 = 100; // the cloth is a square, the minimum is 2
const CLOTH_CENTER_X: f32 = 0.0;
const CLOTH_CENTER_Y: f32 = 15.0;
const CLOTH_CENTER_Z: f32 = 0.0;

struct MyApp {
    camera_bind_group: wgpu::BindGroup,
    // sphere
    sphere_pipeline: wgpu::RenderPipeline,
    sphere_vertex_buffer: wgpu::Buffer,
    sphere_index_buffer: wgpu::Buffer,
    sphere_indices: Vec<u16>,
    // cloth
    cloth_pipeline: wgpu::RenderPipeline,
    cloth_vertex_buffer: wgpu::Buffer,
    cloth_index_buffer: wgpu::Buffer,
    cloth_indices: Vec<u16>,
    // compute
    compute_pipeline: wgpu::ComputePipeline,
    compute_vertices_bind_group: wgpu::BindGroup,
    compute_data_bind_group: wgpu::BindGroup,
    compute_velocities_bind_group: wgpu::BindGroup,
    compute_data_buffer: wgpu::Buffer,
    compute_data: ComputeData,
}

impl MyApp {
    fn new(context: &Context) -> Self {
        let camera = Camera {
            eye: (20.0, 50.0, 75.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: context.get_aspect_ratio(),
            fovy: 45.0,
            znear: 0.1,
            zfar: 1000.0,
        };

        // camera ------------------------------------------------------------
        let (_camera_buffer, camera_bind_group) = camera.create_camera_bind_group(context);

        // sphere ------------------------------------------------------------
        let sphere_pipeline = context.create_render_pipeline(
            "Render Pipeline Sphere",
            include_str!("blue.wgsl"),
            &[Vertex::desc()],
            &[&context.camera_bind_group_layout],
            wgpu::PrimitiveTopology::LineList
        );

        let (mut sphere_vertices, sphere_indices) = icosphere(4);

        // agrandir la sphere :
        for vertex in sphere_vertices.iter_mut() {
            let mut posn = cgmath::Vector3::from(vertex.position);
            posn *= 10.0;
            vertex.position = posn.into()
        }

        // create a buffer for the sphere
        let sphere_vertex_buffer = context.create_buffer(
            &sphere_vertices,
            wgpu::BufferUsages::VERTEX
        );
        let sphere_index_buffer = context.create_buffer(
            &sphere_indices,
            wgpu::BufferUsages::INDEX
        );

        // Cloth ------------------------------------------------------------
        let cloth_pipeline = context.create_render_pipeline(
            "Render Pipeline Cloth",
            include_str!("red.wgsl"),
            &[Vertex::desc()],
            &[&context.camera_bind_group_layout],
            wgpu::PrimitiveTopology::TriangleList
        );

        
        // create the cloth
        let mut cloth_vertices = Vec::new();
        let mut cloth_indices: Vec<u16> = Vec::new();
        
        // create the vertices
        for i in 0..N_CLOTH_VERTICES_PER_ROW {
            for j in 0..N_CLOTH_VERTICES_PER_ROW {
                cloth_vertices.push(Vertex {
                    position: [
                        CLOTH_CENTER_X + (i as f64 * (CLOTH_SIZE as f64 / (N_CLOTH_VERTICES_PER_ROW - 1) as f64)) as f32 - (CLOTH_SIZE / 2) as f32,
                        CLOTH_CENTER_Y,
                        CLOTH_CENTER_Z + (j as f64 * (CLOTH_SIZE as f64 / (N_CLOTH_VERTICES_PER_ROW - 1) as f64)) as f32 - (CLOTH_SIZE / 2) as f32,
                    ],
                    normal: [0.0, 0.0, 0.0],
                    tangent: [0.0, 0.0, 0.0],
                    tex_coords: [0.0, 0.0],
                });
            }
        }
        // create the indices
        for i in 0..N_CLOTH_VERTICES_PER_ROW - 1 {
            for j in 0..N_CLOTH_VERTICES_PER_ROW - 1 {
                // first triangle
                cloth_indices.push((i * N_CLOTH_VERTICES_PER_ROW + j) as u16);
                cloth_indices.push((i * N_CLOTH_VERTICES_PER_ROW + j + 1) as u16);
                cloth_indices.push(((i + 1) * N_CLOTH_VERTICES_PER_ROW + j) as u16);
                // second triangle
                cloth_indices.push((i * N_CLOTH_VERTICES_PER_ROW + j + 1) as u16);
                cloth_indices.push(((i + 1) * N_CLOTH_VERTICES_PER_ROW + j + 1) as u16);
                cloth_indices.push(((i + 1) * N_CLOTH_VERTICES_PER_ROW + j) as u16);
            }
        }
        // set the default speed of the cloth
        let mut cloth_velocities: Vec<Velocity> = Vec::new();
        for _i in cloth_vertices.iter_mut() {
            cloth_velocities.push(Velocity {
                velocity: [0.0, 0.0, 0.0],
            });
        }

        // create a buffer for the cloth
        let cloth_vertex_buffer = context.create_buffer(
            &cloth_vertices,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE
        );
        let cloth_index_buffer = context.create_buffer(
            &cloth_indices,
            wgpu::BufferUsages::INDEX
        );
        let cloth_velocities_buffer = context.create_buffer(
            &cloth_velocities,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX
        );

        // create the compute pipeline
        let compute_pipeline = context.create_compute_pipeline(
            "Compute Pipeline",
            include_str!("compute.wgsl"),
        );

        let compute_vertices_bind_group = context.create_bind_group(
            "compute vertices bind group",
            &compute_pipeline.get_bind_group_layout(0),
            &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: cloth_vertex_buffer.as_entire_binding(),
                },
            ],
        );
        let compute_velocities_bind_group = context.create_bind_group(
            "compute velocities bind group",
            &compute_pipeline.get_bind_group_layout(1),
            &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: cloth_velocities_buffer.as_entire_binding(),
                },
            ],
        );

        let compute_data = ComputeData {
            delta_time: 0.016,
            nb_vertices: N_CLOTH_VERTICES_PER_ROW*N_CLOTH_VERTICES_PER_ROW,
        };

        let compute_data_buffer = context.create_buffer(
            &[compute_data],
            wgpu::BufferUsages::UNIFORM,
        );

        let compute_data_bind_group = context.create_bind_group(
            "compute data bind group",
            &compute_pipeline.get_bind_group_layout(2),
            &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: compute_data_buffer.as_entire_binding(),
                },
            ],
        );

        Self {
            camera_bind_group,
            // sphere
            sphere_pipeline,
            sphere_vertex_buffer,
            sphere_index_buffer,
            sphere_indices,
            // cloth
            cloth_pipeline,
            cloth_vertex_buffer,
            cloth_index_buffer,
            cloth_indices,
            // compute
            compute_pipeline,
            compute_vertices_bind_group,
            compute_velocities_bind_group,
            compute_data_bind_group,
            compute_data_buffer,
            compute_data,
        }
    }
    
}

impl Application for MyApp {
    fn render(&self, context: &Context) -> Result<(), wgpu::SurfaceError> {
        let mut frame = Frame::new(context)?;

        {
            let mut render_pass = frame.begin_render_pass(wgpu::Color {r: 0.85, g: 0.85, b: 0.85, a: 1.0});
            // render the sphere
            render_pass.set_pipeline(&self.sphere_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.sphere_vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.sphere_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.sphere_indices.len() as u32, 0, 0..1);
            // render the cloth as a triangle list
            render_pass.set_pipeline(&self.cloth_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.cloth_vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.cloth_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.cloth_indices.len() as u32, 0, 0..1);
        }

        frame.present();
        
        Ok(())
    }

    fn update(&mut self, context: &Context, delta_time: f32) {
        // update the compute data
        let compute_data = ComputeData {
            delta_time,
            nb_vertices: N_CLOTH_VERTICES_PER_ROW*N_CLOTH_VERTICES_PER_ROW,
        };
        context.update_buffer(&self.compute_data_buffer, &[compute_data]);

        let mut computation = Computation::new(context);

        {
            let mut compute_pass = computation.begin_compute_pass();

            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.compute_vertices_bind_group, &[]);
            compute_pass.set_bind_group(1, &self.compute_velocities_bind_group, &[]);
            compute_pass.set_bind_group(2, &self.compute_data_bind_group, &[]);
            compute_pass.dispatch_workgroups(((N_CLOTH_VERTICES_PER_ROW*N_CLOTH_VERTICES_PER_ROW) as f64/64.0).ceil() as u32, 1, 1);
        }
        computation.submit();
    }
}

fn main() {
    let window = Window::new();

    let context = window.get_context();

    let my_app = MyApp::new(context);

    window.run(my_app);
}