use crate::{serial_print, serial_println};
use conquer_once::spin::OnceCell;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_util::{
    stream::{Stream, StreamExt},
    task::AtomicWaker,
};

static TICK_QUEUE: OnceCell<ArrayQueue<()>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

/// Called by the keyboard interrupt handler
///
/// Must not block or allocate.
pub(crate) fn add_tick() {
    if let Ok(queue) = TICK_QUEUE.try_get() {
        if let Err(_) = queue.push(()) {
            serial_println!("WARNING: tick queue full; dropping keyboard input");
        } else {
            WAKER.wake();
        }
    } else {
        serial_println!("WARNING: tick queue uninitialized");
    }
}

pub struct TickStream {
    _private: (),
}

impl TickStream {
    pub fn new() -> Self {
        TICK_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("TickStream::new should only be called once");
        TickStream { _private: () }
    }
}

impl Stream for TickStream {
    type Item = ();

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<()>> {
        let queue = TICK_QUEUE
            .try_get()
            .expect("tick queue not initialized");

        // fast path
        if let Ok(tick) = queue.pop() {
            return Poll::Ready(Some(tick));
        }

        WAKER.register(&cx.waker());
        match queue.pop() {
            Ok(tick) => {
                WAKER.take();
                Poll::Ready(Some(tick))
            }
            Err(crossbeam_queue::PopError) => Poll::Pending,
        }
    }
}

pub async fn print_ticks() {
    let mut ticks = TickStream::new();

    while let Some(_) = ticks.next().await {
        serial_print!(".");
    }
}

