#![allow(unused)]

use std::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicU8, Ordering::Relaxed},
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
    thread,
    thread::Thread,
};
pub struct Executor {
    future: Pin<Box<dyn Future<Output = ()>>>,
    waker: Waker,
    halt: AtomicU8,
}

impl Executor {
    pub fn new(f: impl Future<Output = ()> + 'static) -> Executor {
        let waker = unsafe { Waker::from_raw(create_raw_waker(thread::current())) };
        let future = unsafe { Pin::new_unchecked(Box::new(f)) };

        let halt = AtomicU8::new(0);

        Executor {
            future,
            waker,
            halt,
        }
    }

    pub fn run(&mut self) {
        let Executor {
            future,
            waker,
            halt,
            ..
        } = self;
        let mut context = Context::from_waker(&waker);

        halt.store(0, Relaxed);

        // Stop execution when we receive the signal
        while halt.load(Relaxed) == 0 {
            match future.as_mut().poll(&mut context) {
                Poll::Pending => thread::park(),
                Poll::Ready(_) => (),
            }
        }

        // halt.fetch_add(1, Relaxed);

        // waker.wake_by_ref();
    }

    pub fn halt<'a>(&'a self) -> Halt<'a> {
        Halt(&self.halt)
    }
}

/// Future that resolves after n polls
pub struct Halt<'a>(&'a AtomicU8);

impl<'a> Future for Halt<'a> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.0.fetch_add(1, Relaxed) {
            0 => Poll::Ready(()),
            _ => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}

fn create_raw_waker(thread: Thread) -> RawWaker {
    RawWaker::new(
        Box::into_raw(Box::new(thread)) as *const _,
        &RawWakerVTable::new(
            |ptr| unsafe { create_raw_waker((&*(ptr as *const Thread)).clone()) },
            |ptr| unsafe {
                Box::from_raw(ptr as *mut Thread).unpark();
            },
            |ptr| unsafe { (&*(ptr as *const Thread)).unpark() },
            |ptr| unsafe {
                Box::from_raw(ptr as *mut Thread);
            },
        ),
    )
}
