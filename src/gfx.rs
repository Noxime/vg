use crate::{
    assets::{Asset, AssetLoader, AssetTy},
    Game, Transform, Vg,
};
use glam::{Vec2, Vec3, Vec4};
use log::*;
use png::ColorType;
use rend3::{
    datatypes::{
        AffineTransform, AlbedoComponent, Camera, CameraProjection, DirectionalLight, Material,
        MeshBuilder, MeshHandle, Object, RendererTextureFormat, Texture, TextureHandle,
    },
    Renderer, RendererBuilder, RendererOptions,
};

use rend3_list::DefaultPipelines;

use serde::{ser::SerializeStruct, Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex},
    unimplemented,
};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Color {
    Value(Vec4),
    Texture(Asset),
}

impl Color {
    pub fn white() -> Color {
        Color::Value(Vec4::splat(1.0))
    }
}

// impl From<Vec4> for Color {
//     fn from(col: Vec4) -> Self {
//         Color::Value(col)
//     }
// }

impl From<Asset> for Color {
    fn from(a: Asset) -> Self {
        Color::Texture(a)
    }
}

impl<P: AsRef<Path>> From<P> for Color {
    fn from(path: P) -> Self {
        Color::Texture(Asset::new(path))
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::white()
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub struct Model {
    pub enabled: bool,
    pub transform: Transform,
    pub asset: Asset,
    pub color: Color,
}

impl<P: AsRef<Path>> From<P> for Model {
    fn from(path: P) -> Model {
        Model {
            asset: Asset::new(path),
            ..Default::default()
        }
    }
}

impl Default for Model {
    fn default() -> Self {
        Model {
            enabled: true,
            transform: Transform::default(),
            asset: Asset::new("#error.obj"),
            color: Color::white(),
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub struct Sprite {
    pub enabled: bool,
    pub transform: Transform,
    pub asset: Asset,
}

impl<P: AsRef<Path>> From<P> for Sprite {
    fn from(path: P) -> Sprite {
        Sprite {
            asset: Asset::new(path),
            ..Default::default()
        }
    }
}

impl Default for Sprite {
    fn default() -> Self {
        Sprite {
            enabled: true,
            transform: Transform::default(),
            asset: Asset::new("#error.png"),
        }
    }
}

struct Drawable {
    mesh: Asset,
    color: Color,
    lit: bool,
    transform: Transform,
}

lazy_static::lazy_static! {
    static ref RENDER_LIST: Mutex<Option<Vec<Drawable>>> = Mutex::new(None);
}

// we implement a custom serializer that will push our model to the render list if drawing is in progress
impl Serialize for Model {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.enabled {
            if let Some(ref mut list) = *RENDER_LIST.lock().unwrap() {
                list.push(Drawable {
                    mesh: self.asset,
                    color: self.color,
                    lit: true,
                    transform: self.transform,
                });
            }
        }

        let mut s = serializer.serialize_struct("Model", 4)?;
        s.serialize_field("enabled", &self.enabled)?;
        s.serialize_field("transform", &self.transform)?;
        s.serialize_field("asset", &self.asset)?;
        s.serialize_field("color", &self.color)?;
        s.end()
    }
}

impl Serialize for Sprite {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.enabled {
            if let Some(ref mut list) = *RENDER_LIST.lock().unwrap() {
                list.push(Drawable {
                    mesh: Asset::sprite(),
                    color: Color::Texture(self.asset),
                    lit: false,
                    transform: self.transform,
                });
            }
        }

        let mut s = serializer.serialize_struct("Sprite", 3)?;
        s.serialize_field("enabled", &self.enabled)?;
        s.serialize_field("transform", &self.transform)?;
        s.serialize_field("asset", &self.asset)?;
        s.end()
    }
}

pub(crate) struct Gfx {
    model_cache: HashMap<Asset, MeshHandle>,
    texture_cache: HashMap<Asset, TextureHandle>,
    renderer: Arc<Renderer>,
    pipelines: DefaultPipelines,
    options: RendererOptions,
}

impl Gfx {
    pub async fn new(window: &winit::window::Window) -> Gfx {
        let window_size = window.inner_size();

        let options = RendererOptions {
            vsync: rend3::VSyncMode::Off,
            size: [window_size.width, window_size.height],
        };

        let renderer = RendererBuilder::new(options.clone())
            .window(window)
            .build()
            .await
            .unwrap();

        info!(
            "Graphics: {} ({:?})",
            renderer.adapter_info().name,
            renderer.mode()
        );

        let shaders = rend3_list::DefaultShaders::new(&renderer).await;
        let pipelines = rend3_list::DefaultPipelines::new(&renderer, &shaders).await;

        renderer.add_directional_light(DirectionalLight {
            color: [1.0; 3].into(),
            intensity: 10.0,
            direction: [1.0, -4.0, -2.0].into(),
        });

        let sprite_model = renderer.add_mesh(
            rend3::datatypes::MeshBuilder::new(vec![
                Vec3::new(1.0, 1.0, 0.0),
                Vec3::new(1.0, -1.0, 0.0),
                Vec3::new(-1.0, 1.0, 0.0),
                Vec3::new(-1.0, -1.0, 0.0),
            ])
            .with_vertex_uvs(vec![
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(0.0, 0.0),
                Vec2::new(0.0, 1.0),
            ])
            .with_indices(vec![2, 3, 1, 2, 1, 0])
            .build(),
        );

        let error_tex = renderer.add_texture_2d(Texture {
            data: vec![
                255, 0, 255, 255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 0, 255, 255,
            ],
            format: RendererTextureFormat::Rgba8Srgb,
            width: 2,
            height: 2,
            label: Some("Missing texture".into()),
            mip_levels: 1,
        });

        Gfx {
            model_cache: std::iter::once((Asset::sprite(), sprite_model)).collect(),
            texture_cache: std::iter::once((Asset::error_tex(), error_tex)).collect(),
            renderer,
            pipelines,
            options,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.options.size = [width, height];
        self.renderer.set_options(self.options.clone());
    }

    pub(crate) async fn draw<G>(
        &mut self,
        vg: &Vg<G>,
        assets: &mut AssetLoader,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        G: Game,
    {
        *RENDER_LIST.lock()? = Some(vec![]);
        let _ = bincode::serialize(&vg.state);
        for player in vg.players() {
            let _ = bincode::serialize(&player.state);
        }

        let drawables = RENDER_LIST.lock()?.take().unwrap_or_default();

        if drawables.is_empty() {
            return Ok(())
        }

        let mut objects = vec![];
        let mut materials = vec![];

        for model in drawables {
            let mesh = self.cached_mesh(assets, model.mesh);
            let albedo = match model.color {
                Color::Value(c) => AlbedoComponent::Value(c),
                Color::Texture(tex) => AlbedoComponent::Texture(self.cached_texture(assets, tex)),
            };

            let material = self.renderer.add_material(Material {
                albedo,
                unlit: !model.lit,
                ..Default::default()
            });

            let object = self.renderer.add_object(Object {
                mesh,
                material,
                transform: AffineTransform {
                    transform: glam::Mat4::from_cols_array(model.transform.to_mat().as_array()),
                },
            });

            materials.push(material);
            objects.push(object);
        }

        // do render
        let render_list = rend3_list::default_render_list(
            self.renderer.mode(),
            self.options.size,
            &self.pipelines,
        );

        self.renderer.set_camera_data(Camera {
            location: [-1.5, 1.5, 2.5].into(),
            projection: CameraProjection::Projection {
                vfov: 70.0,
                near: 0.1,
                pitch: 0.5,
                yaw: 2.59,
            },
        });

        self.renderer
            .render(render_list, rend3::RendererOutput::InternalSwapchain)
            .await;

        for obj in objects {
            self.renderer.remove_object(obj);
        }
        for mat in materials {
            self.renderer.remove_material(mat);
        }

        Ok(())
    }

    fn cached_mesh(&mut self, assets: &mut AssetLoader, asset: Asset) -> MeshHandle {
        if let Some(mesh) = self.model_cache.get(&asset) {
            *mesh
        } else {
            let bytes = assets.load(&asset).unwrap();

            let mesh = match asset.ty {
                AssetTy::Obj => {
                    info!("Loading mesh {:?}", asset);
                    let obj = wavefront::Obj::from_reader(std::io::Cursor::new(bytes)).unwrap();

                    let mut vertex_positions = vec![];
                    let mut vertex_normals = vec![];
                    let mut vertex_uvs = vec![];
                    // let mut vertex_colors = vec![];
                    // let mut vertex_material_indices = vec![];
                    for face in obj.triangles() {
                        for v in face.iter() {
                            vertex_positions.push(v.position().into());
                            vertex_normals.push(v.normal().unwrap_or_default().into());
                            vertex_uvs
                                .push(v.uv().map(|t| [t[0], t[1]]).unwrap_or_default().into());
                            // vertex_colors.push([255; 4]);
                            // vertex_material_indices.push(0);
                        }
                    }

                    let mut mesh = MeshBuilder::new(vertex_positions)
                        .with_vertex_normals(vertex_normals)
                        .with_vertex_uvs(vertex_uvs)
                        // .with_right_handed()
                        .build();

                    if obj.positions().len() != obj.normals().len() {
                        debug!("OBJ does not contain all normals, recalculating all");
                        mesh.calculate_normals();
                    }

                    mesh
                }
                _ => panic!("Only Wavefront OBJ's are supported as meshes"),
            };

            let handle = self.renderer.add_mesh(mesh);
            self.model_cache.insert(asset, handle);
            handle
        }
    }

    fn cached_texture(&mut self, assets: &mut AssetLoader, asset: Asset) -> TextureHandle {
        if let Some(tex) = self.texture_cache.get(&asset) {
            *tex
        } else {
            let bytes = std::io::Cursor::new(assets.load(&asset).unwrap());

            let tex = match asset.ty {
                AssetTy::Png => {
                    info!("Loading texture {:?}", asset);

                    let png = png::Decoder::new(bytes);
                    let (info, mut reader) = png.read_info().unwrap();

                    let mut buf = vec![0; info.buffer_size()];

                    reader.next_frame(&mut buf).unwrap();

                    Texture {
                        data: buf,
                        format: match (info.color_type, info.bit_depth) {
                            (ColorType::RGBA, png::BitDepth::Eight) => {
                                RendererTextureFormat::Rgba8Srgb
                            }
                            (ty, de) => unimplemented!(
                                "Loading textures of type {:?} and bitdepth of {:?} not supported",
                                ty,
                                de
                            ),
                        },
                        width: info.width,
                        height: info.height,
                        label: None,
                        mip_levels: 1,
                    }
                }
                _ => panic!("Only PNG's are supported as textures"),
            };

            let handle = self.renderer.add_texture_2d(tex);
            self.texture_cache.insert(asset, handle);
            handle
        }
    }
}
