use std::sync::Arc;

use wgpu::{
    util::DeviceExt, Adapter, BindGroup, Buffer, Color, CommandEncoderDescriptor, Device,
    FragmentState, Instance, InstanceDescriptor, MultisampleState, Operations,
    PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptionsBase,
    ShaderModuleDescriptor, Surface, TextureFormat, TextureViewDescriptor, VertexState,
    wgt::DeviceDescriptor,
};
use winit::{dpi::PhysicalSize, window::Window};

use crate::shape::ShapeConfig;

pub struct State {
    window: Arc<Window>,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    size: PhysicalSize<u32>,
    surface: Surface<'static>,
    surface_format: TextureFormat,
    // Shape pipeline
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    color_bind_group: BindGroup,
    aspect_buffer: Buffer,
    vertex_count: u32,
    _position_buffer: Buffer,
    // Axis/grid pipeline (optional)
    axis_pipeline: Option<RenderPipeline>,
    axis_vertex_buffer: Option<Buffer>,
    axis_vertex_count: u32,
}

impl State {
    pub async fn new(window: Arc<Window>, config: ShapeConfig) -> State {
        let instance = Instance::new(&InstanceDescriptor::default());

        let adapter = instance
            .request_adapter(&RequestAdapterOptionsBase::default())
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor::default())
            .await
            .unwrap();

        let size = window.inner_size();
        let surface = instance.create_surface(window.clone()).unwrap();
        let cap = surface.get_capabilities(&adapter);
        let surface_format = cap.formats[0];

        // --- Shape vertex buffer ---
        let vertices = config.vertices();
        let vertex_count = vertices.len() as u32;
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // --- Color uniform ---
        let color_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&config.color),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        // --- Aspect ratio uniform (vec4, only .x used) ---
        let aspect = size.width as f32 / size.height as f32;
        let aspect_data: [f32; 4] = [aspect, 0.0, 0.0, 0.0];
        let aspect_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&aspect_data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // --- Position uniform (vec4, only .x and .y used) ---
        let pos_data: [f32; 4] = [config.position[0], config.position[1], 0.0, 0.0];
        let position_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&pos_data),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let uniform_entry = |binding: u32, visibility: wgpu::ShaderStages| wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                uniform_entry(0, wgpu::ShaderStages::FRAGMENT),
                uniform_entry(1, wgpu::ShaderStages::VERTEX),
                uniform_entry(2, wgpu::ShaderStages::VERTEX),
            ],
        });

        let color_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: color_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: aspect_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: position_buffer.as_entire_binding() },
            ],
        });

        // --- Shape shader & pipeline ---
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("shape_shader.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 8,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x2,
                    }],
                }],
            },
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(surface_format.into())],
            }),
            multiview: None,
            cache: None,
        });

        // --- Axis/grid pipeline (optional) ---
        let (axis_pipeline, axis_vertex_buffer, axis_vertex_count) =
            if config.axis || config.axis_grid {
                let axis_verts = crate::axis::generate_vertices(config.axis_arm_len, config.axis_grid, aspect);
                let axis_vertex_count = axis_verts.len() as u32;

                let axis_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&axis_verts),
                    usage: wgpu::BufferUsages::VERTEX,
                });

                let axis_shader = device.create_shader_module(ShaderModuleDescriptor {
                    label: None,
                    source: wgpu::ShaderSource::Wgsl(include_str!("axis_shader.wgsl").into()),
                });

                let axis_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                });

                // Vertex layout: [x, y, r, g, b, a] — stride 24 bytes
                let axis_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&axis_pipeline_layout),
                    vertex: VertexState {
                        module: &axis_shader,
                        entry_point: Some("vs_main"),
                        compilation_options: Default::default(),
                        buffers: &[wgpu::VertexBufferLayout {
                            array_stride: 24,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[
                                wgpu::VertexAttribute {
                                    offset: 0,
                                    shader_location: 0,
                                    format: wgpu::VertexFormat::Float32x2,
                                },
                                wgpu::VertexAttribute {
                                    offset: 8,
                                    shader_location: 1,
                                    format: wgpu::VertexFormat::Float32x4,
                                },
                            ],
                        }],
                    },
                    primitive: PrimitiveState::default(), // TriangleList
                    depth_stencil: None,
                    multisample: MultisampleState::default(),
                    fragment: Some(FragmentState {
                        module: &axis_shader,
                        entry_point: Some("fs_main"),
                        compilation_options: Default::default(),
                        targets: &[Some(surface_format.into())],
                    }),
                    multiview: None,
                    cache: None,
                });

                (Some(axis_pipeline), Some(axis_vertex_buffer), axis_vertex_count)
            } else {
                (None, None, 0)
            };

        let state = State {
            window,
            adapter,
            device,
            queue,
            size,
            surface,
            surface_format,
            render_pipeline,
            vertex_buffer,
            color_bind_group,
            aspect_buffer,
            vertex_count,
            _position_buffer: position_buffer,
            axis_pipeline,
            axis_vertex_buffer,
            axis_vertex_count,
        };

        state.configure_surface();
        state
    }

    pub fn get_window(&self) -> &Window {
        &self.window
    }

    fn configure_surface(&self) {
        let config = self
            .surface
            .get_default_config(&self.adapter, self.size.width, self.size.height)
            .unwrap();
        self.surface.configure(&self.device, &config);
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.size = new_size;
        self.configure_surface();
        let aspect = new_size.width as f32 / new_size.height as f32;
        let aspect_data: [f32; 4] = [aspect, 0.0, 0.0, 0.0];
        self.queue.write_buffer(&self.aspect_buffer, 0, bytemuck::cast_slice(&aspect_data));
    }

    pub fn render(&mut self) {
        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swapchain texture");

        let view = frame.texture.create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: None });

        {
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: wgpu::LoadOp::Clear(Color::WHITE),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Draw axes/grid first so the shape renders on top
            if let (Some(pipeline), Some(buffer)) = (&self.axis_pipeline, &self.axis_vertex_buffer) {
                rpass.set_pipeline(pipeline);
                rpass.set_vertex_buffer(0, buffer.slice(..));
                rpass.draw(0..self.axis_vertex_count, 0..1);
            }

            // Draw shape
            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_bind_group(0, &self.color_bind_group, &[]);
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.draw(0..self.vertex_count, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
        self.window.pre_present_notify();
        frame.present();
    }
}
