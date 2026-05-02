use std::sync::{atomic::AtomicBool};

pub mod uds;
pub mod email;
pub mod app_state;
pub mod queue;
pub mod signaling;
pub mod thread_pool;
pub mod smtp;
pub mod cli;
pub type Closure =
    Box<dyn 'static + Fn(&mut Vec<String>, String)>;

pub static WILL_SHUTDOWN: AtomicBool = AtomicBool::new(false);
