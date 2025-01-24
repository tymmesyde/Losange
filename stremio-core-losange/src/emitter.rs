use std::{fmt::Debug, sync::RwLock};

use relm4::Sender;

type Listener<T> = RwLock<Vec<Box<dyn Fn(&T) + 'static + Send + Sync>>>;

pub struct Emitter<T> {
    listeners: Listener<T>,
}

impl<T> Default for Emitter<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Emitter<T> {
    pub fn new() -> Self {
        Self {
            listeners: RwLock::new(Vec::new()),
        }
    }

    pub fn subscribe<Msg, F>(&self, sender: &Sender<Msg>, f: F)
    where
        F: Fn(&T) -> Option<Msg> + 'static + Send + Sync,
        Msg: Debug + Send + 'static,
    {
        let sender = sender.clone();
        let callback = Box::new(move |event: &T| {
            if let Some(msg) = f(event) {
                sender
                    .send(msg)
                    .expect("Emitter failed to send event to sender");
            }
        });

        self.listeners
            .write()
            .expect("Emitter failed to write")
            .push(callback);
    }

    pub fn emit(&self, event: T) {
        self.listeners
            .read()
            .expect("Emitter failed to read")
            .iter()
            .for_each(|listener| {
                listener(&event);
            });
    }
}
