use alloc::vec;
use alloc::{
    sync::{Arc, Weak},
    vec::Vec,
};
use spin::Mutex;

use crate::channels::spsc::{self};
use crate::channels::{ChannelSizeHint, spsc::SPSC};

pub struct SPMC<T: Clone>(Mutex<Vec<Arc<SPSC<T>>>>);

pub fn create<T: Clone>(size_hint: ChannelSizeHint) -> (Sender<T>, ReceiverFactory<T>) {
    let channel: Arc<SPMC<T>> = Arc::new(SPMC(Mutex::new(vec![])));
    let weak_channel = Arc::downgrade(&channel);
    (Sender(channel), ReceiverFactory(weak_channel, size_hint))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SendResult {
    Sent,
    PartiallyFull,
    Full,
    NoConnections,
}

pub struct Sender<T: Clone>(Arc<SPMC<T>>);

impl<T: Clone> Sender<T> {
    /// Warning: Broadcast cost is `O(subscribers × size_of(T))`.
    ///
    /// Consider using a channel of `Arc<T>` if the performance cost is too big
    pub fn try_send(&self, value: T) -> SendResult {
        let mut any_failed = false;
        let mut any_success = false;
        {
            let mut fibers = self.0.0.lock();

            // This is safe because if all weak references have been dropped,
            // there can never be a new receiver created, so we can drop the fiber
            fibers.retain(|fiber| Arc::weak_count(fiber) > 0);

            for fiber in fibers.iter() {
                if let spsc::SendResultInner::Full = fiber.try_send(value.clone()) {
                    any_failed = true;
                } else {
                    any_success = true;
                }
            }
        }
        match (any_success, any_failed) {
            (false, false) => SendResult::NoConnections, // No fibers in channel
            (true, false) => SendResult::Sent,           // Best case
            (false, true) => SendResult::Full,           // Everything is backed up
            (true, true) => SendResult::PartiallyFull, // Some channels are full, some accepted the message
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReceiverCreationError {
    Closed,
}

pub struct ReceiverFactory<T: Clone>(Weak<SPMC<T>>, ChannelSizeHint);

impl<T: Clone> ReceiverFactory<T> {
    pub fn connect(&self) -> Result<Receiver<T>, ReceiverCreationError> {
        if let Some(arc) = self.0.upgrade() {
            let channel = spsc::new::<T>(self.1);
            let weak = Arc::downgrade(&channel);

            {
                let mut fibers = arc.0.lock();
                fibers.push(channel);
            }

            Ok(Receiver(weak))
        } else {
            Err(ReceiverCreationError::Closed)
        }
    }
}

pub enum ReceiveResult<T> {
    Received(T),
    Empty,
    Closed,
}

pub struct Receiver<T>(Weak<SPSC<T>>);

impl<T> Receiver<T> {
    pub fn try_receive(&self) -> ReceiveResult<T> {
        if let Some(arc) = self.0.upgrade() {
            if let spsc::ReceiveResultInner::Received(result) = arc.try_receive() {
                ReceiveResult::Received(result)
            } else {
                ReceiveResult::Empty
            }
        } else {
            ReceiveResult::Closed
        }
    }
}
