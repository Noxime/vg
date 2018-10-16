use graphics::*;
use ::graphics::hal::Instance;

#[cfg(feature = "backend-gl")]
type GLBack = gfx_backend_gl::Backend;
#[cfg(feature = "backend-vk")]
type VKBack = gfx_backend_vulkan::Backend;
#[cfg(feature = "backend-mt")]
type MTBack = gfx_backend_metal::Backend;
#[cfg(feature = "backend-dx")]
type DXBack = gfx_backend_dx12::Backend;

pub struct GfxBackend<B: hal::Backend> {
    adapter: GfxAdapter<B>,
    surface: B::Surface,
    instance: Option<Box<hal::Instance<Backend = B>>>,
}

// OpenGL is a legacy asshat that uses different init scheme
#[cfg(feature = "backend-gl")]
impl GfxBackend<GLBack> {
    pub fn new_gl(window: &mut Window) -> Result<Self, ()> {
        let window = {
            let builder = gfx_backend_gl::config_context(
                gfx_backend_gl::glutin::ContextBuilder::new(),
                COLOR_FORMAT,
                None,
            )
            .with_vsync(true);
            gfx_backend_gl::glutin::GlWindow::new(
                window.wb.take().ok_or(())?,
                builder,
                &window.events,
            ).map_err(|_| ())?
        };

        let surface = gfx_backend_gl::Surface::from_window(window);
        let mut adapters = surface.enumerate_adapters();

        Ok(GfxBackend {
            adapter: GfxAdapter::new(&mut adapters)?,
            surface,
            instance: None,
        })
    }

    pub fn info(&self) -> String {
        format!("OpenGL {}", self.adapter.info())
    }
}

#[cfg(feature = "backend-vk")]
impl GfxBackend<VKBack> {
    pub fn new_vk(window: &mut Window) -> Result<Self, ()> {
        let window = window.wb.take().ok_or(())?
            .build(&window.events)
            .map_err(|_| ())?;
        let instance = gfx_backend_vulkan::Instance::create("kea vulkan", 1);
        let surface = instance.create_surface(&window);
        let mut adapters = instance.enumerate_adapters();
        Ok(GfxBackend {
            adapter: GfxAdapter::new(&mut adapters)?,
            surface,
            instance: Some(Box::new(instance)),
        })
    }

    pub fn info(&self) -> String {
        format!("Vulkan {}", self.adapter.info())
    }
}

#[cfg(feature = "backend-mt")]
impl GfxBackend<MTBack> {
    pub fn new_mt(window: &mut Window) -> Result<Self, ()> {
        let window = window.wb.take().ok_or(())?
            .build(&window.events)
            .map_err(|_| ())?;
        let instance = gfx_backend_metal::Instance::create("kea metal", 1);
        let surface = instance.create_surface(&window);
        let mut adapters = instance.enumerate_adapters();
        Ok(GfxBackend {
            adapter: GfxAdapter::new(&mut adapters)?,
            surface,
            instance: Some(Box::new(instance)),
        })
    }

    pub fn info(&self) -> String {
        format!("Metal {}", self.adapter.info())
    }
}

#[cfg(feature = "backend-dx")]
impl GfxBackend<DXBack> {
    pub fn new_dx(window: &mut Window) -> Result<Self, ()> {
        let window = window.wb.take().ok_or(())?
            .build(&window.events)
            .map_err(|_| ())?;
        let instance = gfx_backend_dx12::Instance::create("kea dx12", 1);
        let surface = instance.create_surface(&window);
        let mut adapters = instance.enumerate_adapters();
        Ok(GfxBackend {
            adapter: GfxAdapter::new(&mut adapters)?,
            surface,
            instance: Some(Box::new(instance)),
        })
    }

    pub fn info(&self) -> String {
        format!("DirectX 12 {}", self.adapter.info())
    }
}
