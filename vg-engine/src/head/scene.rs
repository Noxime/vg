use crate::prelude::*;
use std::sync::Arc;

// use rend3::{
//     graph::ViewportRect,
//     types::{Handedness, SampleCount},
//     *,
// };
// use rend3_routine::{base::BaseRenderGraph, pbr::PbrRoutine, tonemapping::TonemappingRoutine};
use wgpu::{Adapter, Device, Instance, Queue, Texture, TextureFormat};

pub struct Scene {
    // renderer: Arc<Renderer>,
    // spp: ShaderPreProcessor,
    // render_graph: BaseRenderGraph,
    // pbr: PbrRoutine,
    // tonemapping: TonemappingRoutine,
}

#[profile_all]
impl Scene {
    pub fn new(
        instance: Arc<Instance>,
        adapter: Arc<Adapter>,
        device: Arc<Device>,
        queue: Arc<Queue>,
    ) -> Result<Self> {
        // let iad = InstanceAdapterDevice {
        //     info: adapter.get_info().into(),
        //     instance,
        //     adapter,
        //     device,
        //     queue,
        //     profile: RendererProfile::CpuDriven,
        // };

        // // First try to create a GPU driven renderer, fallback to CpuDriven before erroring out
        // let renderer;
        // if let Ok(r) = Renderer::new(iad.clone(), Handedness::Left, None) {
        //     renderer = r;
        // } else {
        //     renderer = Renderer::new(
        //         InstanceAdapterDevice {
        //             profile: RendererProfile::CpuDriven,
        //             ..iad
        //         },
        //         Handedness::Left,
        //         None,
        //     )?;
        // }

        // info!(profile = ?renderer.profile, "Initialized rend3");

        // // Add default shaders to the shader "source"
        // let mut spp = ShaderPreProcessor::new();
        // rend3_routine::builtin_shaders(&mut spp);

        // // The format is a placeholder, will get recreated when Head::configure is called
        // let (render_graph, pbr, tonemapping) =
        //     create_graph(&renderer, &spp, TextureFormat::Rgba8Unorm);

        // Ok(Self {
        //     renderer,
        //     spp,
        //     render_graph,
        //     pbr,
        //     tonemapping,
        // })

        Ok(Self {})
    }

    pub fn configure(&mut self, format: TextureFormat) {
        // let (graph, pbr, tonemap) = create_graph(&self.renderer, &self.spp, format);
        // self.render_graph = graph;
        // self.pbr = pbr;
        // self.tonemapping = tonemap;
    }

    /// Render 3D content
    pub fn render(&self, surface: &Texture) {
        /*
        let resolution = UVec2::new(surface.width(), surface.height());

        // Swap the instruction buffers so that our frame's changes can be processed.
        self.renderer.swap_instruction_buffers();
        // Evaluate our frame's world-change instructions
        let mut eval_output = self.renderer.evaluate_instructions();

        // Build a rendergraph
        let mut graph = rend3::graph::RenderGraph::new();

        // Import the surface texture into the render graph.
        let frame_handle = graph.add_imported_render_target(
            surface,
            0..1,
            0..1,
            ViewportRect::from_size(resolution),
        );
        // Add the default rendergraph without a skybox
        self.render_graph.add_to_graph(
            &mut graph,
            rend3_routine::base::BaseRenderGraphInputs {
                eval_output: &eval_output,
                routines: rend3_routine::base::BaseRenderGraphRoutines {
                    pbr: &self.pbr,
                    skybox: None,
                    tonemapping: &self.tonemapping,
                },
                target: rend3_routine::base::OutputRenderTarget {
                    handle: frame_handle,
                    resolution,
                    samples: SampleCount::One,
                },
            },
            rend3_routine::base::BaseRenderGraphSettings {
                ambient_color: glam::Vec4::ZERO,
                clear_color: glam::Vec4::new(0.10, 0.05, 0.10, 1.0), // Nice scene-referred purple
            },
        );

        // Dispatch a render using the built up rendergraph!
        graph.execute(&self.renderer, &mut eval_output);
         */
    }
}

/*
/// (Re)create the render graph for some texture format
fn create_graph(
    renderer: &Arc<Renderer>,
    spp: &ShaderPreProcessor,
    format: TextureFormat,
) -> (BaseRenderGraph, PbrRoutine, TonemappingRoutine) {
    let graph = BaseRenderGraph::new(&renderer, &spp);

    let mut data_core = renderer.data_core.lock();
    let pbr_routine = rend3_routine::pbr::PbrRoutine::new(
        &renderer,
        &mut data_core,
        &spp,
        &graph.interfaces,
        &graph.gpu_culler.culling_buffer_map_handle,
    );
    drop(data_core);
    let tonemapping_routine = rend3_routine::tonemapping::TonemappingRoutine::new(
        &renderer,
        &spp,
        &graph.interfaces,
        format,
    );

    (graph, pbr_routine, tonemapping_routine)
}
 */