use std::sync::atomic::{AtomicBool, AtomicU64};

#[derive(Debug)]
struct AppState {
    is_shutdown : AtomicBool,
    is_processing : AtomicU64

}


impl Default for AppState {
    fn default() -> Self {
        Self { 
            is_shutdown: AtomicBool::new(false),
            is_processing: AtomicU64::new(0) }
        }
}