// #![feature(set_stdio)]

use std::rc::Rc;
use std::sync::Mutex;
use stdweb::console;
use stdweb::unstable::TryInto;
use stdweb::web::{self, html_element::CanvasElement, INode};

mod gfx;

// The default allocator is quite big so this will make release binaries
// smaller as size is a proper issue on the web
#[cfg_attr(not(debug_assertions), global_allocator)]
#[cfg(not(debug_assertions))]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

struct Vg {
    gfx: gfx::Gfx,
    sfx: placeholder_audio::Audio,
    input: vg_gilrs::Input,
}

impl vg::Api for Vg {
    type R = gfx::Gfx;
    type I = vg_gilrs::Input;
    type A = placeholder_audio::Audio;
    type T = Time;

    fn poll(&mut self) {}
    fn exit(&self) -> bool {
        false
    }

    fn audio(&mut self) -> &mut Self::A {
        &mut self.sfx
    }

    fn input(&mut self) -> &mut Self::I {
        &mut self.input
    }

    fn renderer(&mut self) -> &mut Self::R {
        &mut self.gfx
    }
}

struct Time(f64);
impl vg::Time for Time {
    fn new() -> Self {
        Time(web::Date::now())
    }
    fn now(&self) -> f32 {
        ((web::Date::now() - self.0) / 1000.0) as f32
    }
}

pub fn main() {
    // web panics are garbage by default
    std::panic::set_hook(Box::new(|info| {
        console!(error, format!("{}", info));
    }));

    console!(log, "vg start");

    let document = web::document();
    document.set_title("vg");
    let element = document.create_element("canvas").unwrap();
    let canvas: CanvasElement = element.try_into().unwrap();
    let body = document.body().unwrap();
    body.append_child(&canvas);
    canvas.set_width(800);
    canvas.set_height(600);

    let (gfx, waker) = gfx::Gfx::new(canvas);

    let vg = Vg {
        gfx,
        sfx: placeholder_audio::Audio::new(),
        input: vg_gilrs::Input::new(0.0),
    };

    use futures::executor::LocalPool;
    use futures::task::LocalSpawn;
    let executor = LocalPool::new();

    executor
        .spawner()
        .spawn_local_obj(Box::new(game::run(vg)).into())
        .expect("Failed to spawn");

    fn main_loop(mut executor: LocalPool, waker: Rc<Mutex<Option<std::task::Waker>>>) {
        executor.run_until_stalled();

        if let Some(waker) = waker.lock().expect("failed to lock").take() {
            waker.wake();
        } else {
            console!(error, "lol our waker is gone? yikes");
        }

        web::window().request_animation_frame(move |_| main_loop(executor, waker));
    }

    main_loop(executor, waker);
}
