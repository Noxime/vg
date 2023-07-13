use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use async_notify::Notify;
use futures_channel::oneshot;
use futures_executor::LocalPool;
use futures_util::task::LocalSpawnExt;
use vg_interface::WaitReason;

use std::future::Future;

thread_local! {
    static POOL: RefCell<LocalPool> = RefCell::new(LocalPool::new());
    static STALL: Rc<Notify> = Rc::new(Notify::new());
    static WAIT_REASON: Cell<WaitReason> = Cell::new(WaitReason::Startup);
}

/// Yield into the vg runtime, continuing on the next step
pub async fn wait(reason: WaitReason) {
    let notify = STALL.with(|n| n.clone());
    WAIT_REASON.with(|c| c.set(reason));
    notify.notified().await;
}

/// Start the main future, wrapping it in an exit handler
#[doc(hidden)]
pub fn start(future: impl Future<Output = ()> + 'static) {
    spawn(async {
        // Notify the runtime that we are ready to execute
        wait(WaitReason::Startup).await;

        future.await;

        // DEBUG
        println!("Clean exit");
    });
}

/// Execute a step
#[doc(hidden)]
pub fn step() -> WaitReason {
    // Clear the runtime yield
    STALL.with(|notify| notify.notify());

    // Run until something stalls the runtime again
    POOL.with(|pool| pool.borrow_mut().run_until_stalled());
    WAIT_REASON.with(Cell::get)
}

/// Handle to a spawned future that can be joined by awaiting
pub struct JoinHandle<T> {
    receiver: oneshot::Receiver<T>,
}

impl<T> Future for JoinHandle<T> {
    type Output = T;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let receiver = &mut self.get_mut().receiver;
        futures_util::pin_mut!(receiver);
        receiver.poll(cx).map(|r| r.expect("uh oh?"))
    }
}

/// Spawn a future that will execute asynchronously, optionally awaiting on the JoinHandle
pub fn spawn<T: 'static>(future: impl Future<Output = T> + 'static) -> JoinHandle<T> {
    let (sender, receiver) = oneshot::channel();

    POOL.with(|pool| {
        pool.borrow()
            .spawner()
            .spawn_local(async {
                // Caller is free to drop the join handle if they don't care about it
                let _ = sender.send(future.await);
            })
            .unwrap();
    });

    JoinHandle { receiver }
}
