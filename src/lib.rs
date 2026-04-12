use std::{cell::RefCell, sync::{LazyLock, Mutex, atomic::AtomicBool}};

use crate::app_state::AppState;

pub mod uds;
pub mod email;
pub mod app_state;
pub mod queue;

pub static WILL_SHUTDOWN: AtomicBool = AtomicBool::new(false);
