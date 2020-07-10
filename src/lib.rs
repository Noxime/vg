//! VG game engine

pub use log::{debug, error, info, trace, warn};
use std::io::Cursor;
pub use vek;
use zip::ZipArchive;

pub use vg_derive::game;

pub mod asset;
pub mod gfx;
pub mod input;
use asset::Asset;

mod event;
mod time;
pub use event::Event;

#[cfg(not(target_arch = "wasm32"))]
use emoji_logger::init as logger;
#[cfg(target_arch = "wasm32")]
use {
    std::{sync::mpsc, task::Waker},
    stdweb_logger::init as logger,
};

pub type Color = vek::Rgba<f32>;
pub type Size = vek::Extent2<u32>;
pub type Mat = vek::Mat4<f32>;
pub type Pos = vek::Vec2<f32>;

pub struct Vg {
    #[cfg(target_arch = "wasm32")]
    waker: mpsc::SyncSender<Waker>,

    input: input::Input,
    gfx: gfx::Gfx,
    // TODO: Streaming bundles
    assets: ZipArchive<Cursor<&'static [u8]>>,
    evs: futures::channel::mpsc::Receiver<Event>,
    runtime: f64,
    last_frame: f64,
    delta: f64,
}

impl Vg {
    // load an asset
    pub fn asset(&mut self, path: impl AsRef<str>) -> Option<Asset> {
        if let Ok(mut file) = self.assets.by_name(path.as_ref()) {
            use std::io::Read;
            let mut bytes = vec![];
            file.read_to_end(&mut bytes).unwrap();
            Some(Asset { bytes })
        } else {
            None
        }
    }

    pub fn event(&mut self) -> Option<Event> {
        if let Some(event) = self.evs.try_next().ok().flatten() {
            self.gfx.handle(&event);
            self.input.handle(&event);

            Some(event)
        } else {
            None
        }
    }

    pub fn input(&self) -> &input::Input {
        &self.input
    }

    pub async fn texture(&self, source: impl gfx::Source + 'static) -> gfx::Texture {
        let d = source.load().await;
        gfx::Texture {
            source: Box::new(source),
            tex: self.gfx.texture(d),
        }
    }

    pub fn fill(&self, color: Color) {
        self.gfx.fill(color)
    }

    pub fn draw(&self, texture: &gfx::Texture, instances: &[Mat]) {
        self.gfx.draw(texture, instances)
    }

    pub async fn present(&mut self) {
        self.gfx.present();

        // we need to yield execution back to JS on web, so use this custom
        // future to stall the executor until next iteration
        #[cfg(target_arch = "wasm32")]
        {
            use std::future::Future;
            use std::pin::Pin;
            use std::task::{Context, Poll};
            struct Fut(bool, mpsc::SyncSender<Waker>);
            impl Future for Fut {
                type Output = ();
                fn poll(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<()> {
                    if self.0 {
                        Poll::Ready(())
                    } else {
                        self.0 = true;
                        self.1.send(ctx.waker().clone()).unwrap();
                        Poll::Pending
                    }
                }
            }

            Fut(false, self.waker.clone()).await
        }

        // update timings
        let now = time::now();
        self.delta = now - self.last_frame;
        self.runtime += self.delta;
        self.last_frame = now;

        // update keyboard state
        self.input.frame();
    }

    pub fn audio(&mut self, f: impl Fn(&mut [u16])) {

    }
}

// main entrypoint, handles executors
#[doc(hidden)]
pub fn __startup<F, T>(f: F, assets: &'static [u8])
where
    F: Fn(Vg) -> T,
    T: 'static + std::future::Future<Output = ()> + Send,
{
    // init logging, emojilogger/stdweblogger
    logger();

    info!("VG v{}", env!("CARGO_PKG_VERSION"));

    let assets = ZipArchive::new(Cursor::new(assets)).expect("Asset bundle is invalid");
    debug!(
        "Assets: \"{}\"",
        String::from_utf8(assets.comment().into()).expect("Asset comment not utf8")
    );

    let input = input::Input::new();

    let (mut events, evs) = futures::channel::mpsc::channel(16); // do we need bigger buffer? Event's not particularily big

    #[cfg(not(target_arch = "wasm32"))]
    {
        let ev_loop = winit::event_loop::EventLoop::new();
        let window = winit::window::Window::new(&ev_loop).unwrap();
        let gfx = futures::executor::block_on(gfx::Gfx::new(window));

        let vg = Vg {
            assets,
            input,
            gfx,
            evs,
            runtime: 0.0,
            last_frame: time::now(),
            delta: 0.0,
        };
        use futures::executor::ThreadPoolBuilder;
        ThreadPoolBuilder::new()
            .name_prefix("vg-")
            .create()
            .expect("Executor failed to run")
            .spawn_ok(f(vg));

        ev_loop.run(move |event, _, flow| {
            use winit::{
                event::{
                    ElementState, Event as E, KeyboardInput as K, MouseButton, MouseScrollDelta,
                    WindowEvent as W,
                },
                event_loop::ControlFlow,
            };
            *flow = ControlFlow::Wait;
            // trace!("{:#?}", event);
            if let Err(e) = events.try_send(match event {
                E::WindowEvent { event, .. } => match event {
                    W::CloseRequested => {
                        // TODO: Handle more gracefully, currently segfaults?
                        *flow = ControlFlow::Exit;
                        Event::Exit
                    }
                    W::Focused(true) => Event::FocusGained,
                    W::Focused(false) => Event::FocusLost,
                    W::Resized(size) => Event::Resize([size.width, size.height].into()),

                    // kb events
                    W::ReceivedCharacter(c) => Event::Keyboard(input::keyboard::Event::Text(c)),
                    W::KeyboardInput {
                        input:
                            K {
                                state,
                                virtual_keycode: Some(kc),
                                ..
                            },
                        ..
                    } => Event::Keyboard(match state {
                        ElementState::Pressed => {
                            input::keyboard::Event::Down(input::keyboard::winit2key(kc))
                        }
                        ElementState::Released => {
                            input::keyboard::Event::Up(input::keyboard::winit2key(kc))
                        }
                    }),

                    // mouse
                    W::CursorMoved {
                        position, // TODO: Fix coordinates
                        ..
                    } => Event::Mouse(input::mouse::Event::Moved(
                        [position.x as f32, position.y as f32].into(),
                    )),
                    W::CursorEntered { .. } => Event::Mouse(input::mouse::Event::Enter),
                    W::CursorLeft { .. } => Event::Mouse(input::mouse::Event::Leave),
                    W::MouseInput { button, state, .. } => {
                        let button = match button {
                            MouseButton::Left => input::mouse::Button::Left,
                            MouseButton::Right => input::mouse::Button::Right,
                            MouseButton::Middle => input::mouse::Button::Middle,
                            _ => return,
                        };
                        Event::Mouse(match state {
                            ElementState::Pressed => input::mouse::Event::Down(button),
                            ElementState::Released => input::mouse::Event::Up(button),
                        })
                    }
                    W::MouseWheel { delta, .. } => {
                        Event::Mouse(input::mouse::Event::Scroll(match delta {
                            MouseScrollDelta::LineDelta(_, y) => y,
                            MouseScrollDelta::PixelDelta(p) => p.y as f32,
                        }))
                    }

                    _ => return,
                },
                _ => return,
            }) {
                error!("Failed to send event: {}", e);
            }
        })
    }

    #[cfg(target_arch = "wasm32")]
    {
        use stdweb::{
            traits::*, unstable::TryInto, web, web::event::*, web::html_element::CanvasElement,
        };
        std::panic::set_hook(Box::new(|info| {
            error!("PANIC: {}", info);
        }));

        let doc = web::document();

        // try to find #vg-canvas, failing that insert a new one
        let canvas: CanvasElement = match doc.get_element_by_id("vg-canvas") {
            Some(canvas) => canvas,
            None => {
                warn!("'#vg-canvas' not found, inserting new element. Check your spelling");
                let can = doc
                    .create_element("canvas")
                    .expect("Couldn't create canvas");
                doc.body().unwrap().append_child(&can);
                can
            }
        }
        .try_into()
        .expect("Element was not a canvas");

        canvas.set_attribute("tabindex", "0").unwrap();
        canvas.focus();

        let mut e1 = events.clone();
        let mut e2 = events.clone();
        let mut e3 = events.clone();
        let mut e4 = events.clone();
        let mut e5 = events.clone();

        canvas.add_event_listener(move |event: BeforeUnloadEvent| {
            e1.try_send(Event::Exit).unwrap();
        });

        canvas.add_event_listener(move |event: FocusEvent| {
            debug!("Focused");
            e2.try_send(Event::FocusGained).unwrap();
        });

        canvas.add_event_listener(move |event: BlurEvent| {
            debug!("Lost focus");
            e3.try_send(Event::FocusLost).unwrap();
        });

        canvas.add_event_listener(move |event: KeyDownEvent| {
            if event.repeat() {
                return;
            }
            if let Some(key) = input::keyboard::web2key(event.code().as_str()) {
                e4.try_send(Event::Keyboard(input::keyboard::Event::Down(key)))
                    .unwrap();
            }
        });

        canvas.add_event_listener(move |event: KeyUpEvent| {
            if event.repeat() {
                return;
            }
            if let Some(key) = input::keyboard::web2key(event.code().as_str()) {
                e5.try_send(Event::Keyboard(input::keyboard::Event::Up(key)))
                    .unwrap();
            }
        });

        // there shouldn't be more than 1 waker in the channel at any time but
        // make the buffer a little bigger, just in case. Wakers arent very
        // big objects anyway
        let (waker, waker_recv) = mpsc::sync_channel(2);
        let gfx = futures::executor::block_on(gfx::Gfx::new(canvas));
        let vg = Vg {
            waker,
            assets,
            input,
            gfx,
            evs,
            runtime: 0.0,
            last_frame: time::now(),
            delta: 0.0,
        };

        // WASM doesnt support threading
        use futures::executor::LocalPool;
        use futures::task::LocalSpawnExt;
        let exec = LocalPool::new();

        exec.spawner()
            .spawn_local(f(vg))
            .expect("Game future failed to run");

        // execute until present is called
        fn frame(mut exec: LocalPool, waker: mpsc::Receiver<Waker>) {
            exec.run_until_stalled();

            // tell the Present future its okay to return Ready
            if let Ok(waker) = waker.recv() {
                waker.wake()
            }

            // repeat on next frame
            stdweb::web::window().request_animation_frame(move |_| frame(exec, waker));
        }

        frame(exec, waker_recv);
    }
}
