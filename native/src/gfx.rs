use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Instant};

use glam::{vec2, vec3, Mat4, UVec2, Vec3A};
use rend3::{
    create_iad,
    types::{
        AlbedoComponent, Camera, Material, Mesh, MeshHandle, MipmapCount, MipmapSource, Object,
        ObjectHandle, Texture, TextureHandle,
    },
    util::output::OutputFrame,
    Renderer,
};
use rend3_pbr::{PbrRenderRoutine, RenderTextureOptions, SampleCount};
use tracing::*;
use vg_types::Transform;
use wgpu::{
    CommandEncoderDescriptor, Maintain, PresentMode, Surface, TextureFormat, TextureViewDescriptor,
};
use winit::window::Window;

use crate::{assets::Cache, debug::DebugUi};

pub struct Gfx {
    surface: Surface,
    format: TextureFormat,
    present_mode: PresentMode,
    window: Arc<Window>,
    renderer: Arc<Renderer>,
    routine_pbr: PbrRenderRoutine,
    sprite_mesh: MeshHandle,
    textures: HashMap<PathBuf, TextureHandle>,
    sprites: Vec<ObjectHandle>,

    egui_pass: egui_wgpu_backend::RenderPass,
}

impl Gfx {
    pub async fn new(window: Arc<Window>) -> Gfx {
        // //wgpu_subscriber::initialize_default_subscriber(Some(std::path::Path::new("wgpu_trace")));

        let iad = create_iad(None, None, None).await.unwrap();

        info!(
            "{} ({:?}/{:?})",
            iad.info.name, iad.info.backend, iad.info.device_type
        );

        // Lets hope winit never provides a busted window handle and the proper swapchain format can always be queried
        let surface = unsafe { iad.instance.create_surface(window.as_ref()) };
        let format = surface.get_preferred_format(&iad.adapter).unwrap();
        let present_mode = PresentMode::Mailbox;

        let size = window.inner_size();
        let size = UVec2::new(size.width, size.height);

        let renderer = Renderer::new(iad, Some(size.x as f32 / size.y as f32)).unwrap();

        let mut mesh = Mesh {
            vertex_positions: vec![
                vec3(-0.5, -0.5, 0.0),
                vec3(-0.5, 0.5, 0.0),
                vec3(0.5, -0.5, 0.0),
                vec3(0.5, 0.5, 0.0),
            ],
            vertex_uvs: vec![
                vec2(0.0, 1.0),
                vec2(0.0, 0.0),
                vec2(1.0, 1.0),
                vec2(1.0, 0.0),
            ],
            vertex_normals: vec![Default::default(); 4],
            vertex_tangents: vec![Default::default(); 4],
            vertex_colors: vec![[0xFF; 4]; 4],
            vertex_material_indices: vec![0; 4],
            indices: vec![0, 1, 2, 3, 2, 1],
        };

        mesh.calculate_normals();
        mesh.calculate_tangents();

        let sprite_mesh = renderer.add_mesh(mesh);

        let routine_pbr = PbrRenderRoutine::new(
            renderer.as_ref(),
            RenderTextureOptions {
                resolution: size,
                samples: SampleCount::Four,
            },
            format,
        );

        let mut gfx = Gfx {
            egui_pass: egui_wgpu_backend::RenderPass::new(&renderer.device, format, 1),

            surface,
            format,
            present_mode,
            window,
            renderer,
            routine_pbr,
            sprite_mesh,

            textures: HashMap::new(),
            sprites: Vec::new(),
        };

        gfx.resize(size);

        gfx
    }

    pub fn resize(&mut self, size: UVec2) {
        debug!("Resizing to {}x{}", size.x, size.y);

        // Cannot resize to zero
        if size.x == 0 || size.y == 0 {
            return;
        }

        rend3::configure_surface(
            &self.surface,
            &self.renderer.device,
            self.format,
            size,
            self.present_mode,
        );

        let aspect = size.x as f32 / size.y as f32;

        self.renderer.set_aspect_ratio(aspect);

        self.renderer.set_camera_data(Camera {
            projection: rend3::types::CameraProjection::Orthographic {
                size: Vec3A::new(10.0 * aspect, 10.0, 10.0),
                direction: Vec3A::new(0.0, 0.0, 1.0),
            },
            location: Vec3A::new(0.0, 0.0, -5.0),
        });

        self.routine_pbr.resize(
            self.renderer.as_ref(),
            RenderTextureOptions {
                resolution: size,
                samples: SampleCount::Four,
            },
        );
    }

    pub async fn draw_sprite(&mut self, asset: Arc<Cache>, transform: Transform) {
        puffin::profile_function!();

        if !self.textures.contains_key(&asset.path) {
            let bytes = asset.load_all().await;

            let img = image::load_from_memory(&bytes).unwrap();
            let img = img.to_rgba8();

            let tex = self.renderer.add_texture_2d(Texture {
                label: Some(format!("Sprite: {}", asset.path.display())),
                size: UVec2::new(img.width(), img.height()),
                data: img.into_raw(),
                format: TextureFormat::Rgba8UnormSrgb,
                mip_count: MipmapCount::Maximum,
                mip_source: MipmapSource::Generated,
            });

            self.textures.insert(asset.path.clone(), tex);
            debug!("Texture uploaded");
        }

        let tex = self.textures.get(&asset.path).unwrap().clone();

        let material = self.renderer.add_material(Material {
            albedo: AlbedoComponent::Texture(tex),
            unlit: true,
            ..Default::default()
        });
        let obj = self.renderer.add_object(Object {
            mesh: self.sprite_mesh.clone(),
            material: material,
            transform: trans2mat(transform),
        });

        self.sprites.push(obj);
    }

    pub async fn present(&mut self, debug: &mut DebugUi) {
        puffin::profile_function!();

        {
            puffin::profile_scope!("maintain");
            self.renderer.device.poll(Maintain::Poll);
        }

        let frame = match self.surface.get_current_frame() {
            Ok(f) => f,
            Err(e) => {
                warn!("Failed to get output frame from surface: {}", e);
                let size = self.window.inner_size();
                self.resize(UVec2::new(size.width, size.height));
                self.surface.get_current_frame().unwrap()
            }
        };
        let view = Arc::new(
            frame
                .output
                .texture
                .create_view(&TextureViewDescriptor::default()),
        );

        {
            puffin::profile_scope!("rend3_pbr");
            self.renderer
                .render(&mut self.routine_pbr, OutputFrame::View(Arc::clone(&view)));
        }

        {
            puffin::profile_scope!("rend3_clear_scene");
            self.sprites.clear();
        }

        if debug.visible {
            puffin::profile_scope!("egui_render");

            let egui_start = Instant::now();
            debug.platform.begin_frame();

            let mut app_output = epi::backend::AppOutput::default();

            let mut egui_frame = epi::backend::FrameBuilder {
                info: epi::IntegrationInfo {
                    web_info: None,
                    cpu_usage: debug.last_frame_time,
                    seconds_since_midnight: None,
                    native_pixels_per_point: Some(self.window.scale_factor() as f32),
                    prefer_dark_mode: None,
                },
                tex_allocator: &mut self.egui_pass,
                output: &mut app_output,
                repaint_signal: debug.repaint_signal.clone(),
            }
            .build();

            use epi::App;
            debug.update(&debug.platform.context(), &mut egui_frame);

            let (_out, paint_commands) = debug.platform.end_frame(Some(&self.window));
            let paint_jobs = debug.platform.context().tessellate(paint_commands);

            let frame_time = (Instant::now() - egui_start).as_secs_f32();
            debug.last_frame_time = Some(frame_time);

            let mut enc = self
                .renderer
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("egui-encoder"),
                });

            let size = self.window.inner_size();

            let screen_descriptor = egui_wgpu_backend::ScreenDescriptor {
                physical_width: size.width,
                physical_height: size.height,
                scale_factor: self.window.scale_factor() as f32,
            };

            self.egui_pass.update_texture(
                &self.renderer.device,
                &self.renderer.queue,
                // debug.platform.context().texture(),
                &debug.platform.context().texture(),
            );
            self.egui_pass
                .update_user_textures(&self.renderer.device, &self.renderer.queue);
            self.egui_pass.update_buffers(
                &self.renderer.device,
                &self.renderer.queue,
                &paint_jobs,
                &screen_descriptor,
            );
            self.egui_pass
                .execute(&mut enc, &view, &paint_jobs, &screen_descriptor, None)
                .unwrap();

            // Draw the debug UI
            self.renderer.queue.submit(Some(enc.finish()));
        }

        {
            puffin::profile_scope!("wgpu_present");
            // View must be dropped before frame
            drop(view);
            drop(frame);
        }

        // Done with the frame, record it on the profiler
        {
            puffin::profile_scope!("puffin_frame");
            puffin::GlobalProfiler::lock().new_frame();
        }
    }
}

fn trans2mat(trans: Transform) -> Mat4 {
    Mat4::from_scale_rotation_translation(
        trans.scale.into(),
        glam::Quat::from_array(trans.rotation),
        trans.position.into(),
    )
}
