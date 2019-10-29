use vg::renderer::*;

use libnx_rs::libnx;

pub struct SwitchRenderer {
    size: (u16, u16),
    framebuffer: *mut [u8; 4],
}

impl SwitchRenderer {
    pub fn new() -> SwitchRenderer {
        unsafe {
            libnx::gfxInitDefault();
            let mut width = 0u32;
            let mut height = 0u32;
            let fb = libnx::gfxGetFramebuffer(&mut width as *mut u32, &mut height as *mut u32) as *mut u32 as *mut [u8; 4];

            Self {
                size: (width as u16, height as u16),
                framebuffer: fb,
            }
        }
    }
}

impl Drop for SwitchRenderer {
    fn drop(&mut self) {
        unsafe {
            libnx::gfxExit();
        }
    }
}


impl Renderer for SwitchRenderer {
    const NAME: &'static str = "Software";

    type Frame = SwitchFrame;

    fn frame(&mut self, base: Color) -> Self::Frame {
        SwitchFrame {
            size: self.size,
            fb: self.framebuffer,
            base,
        }
    }
}

pub struct SwitchFrame {
    size: (u16, u16),
    fb: *mut [u8; 4],
    base: Color,
}

impl Frame for SwitchFrame {
    fn present(self, vsync: bool) {
        for x in 0 .. 32 {
            for y in 0 .. 32 {
                unsafe {
                    *(self.fb.offset(y as isize * self.size.0 as isize + x as isize)) = [
                        (self.base[0] * 255.0) as u8,
                        (self.base[1] * 255.0) as u8,
                        (self.base[2] * 255.0) as u8,
                        (self.base[3] * 255.0) as u8,
                    ];
                }
            }
        }

        unsafe {
            libnx::gfxFlushBuffers();
            libnx::gfxSwapBuffers();
            if vsync { libnx::gfxWaitForVsync(); }
        }
    }
}