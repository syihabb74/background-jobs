use crate::{WILL_SHUTDOWN, email::Email, queue::Queue};

#[derive(Debug)]
pub struct AppState {
    has_works: bool,
    total_works: u32,
    queue: Queue,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            has_works: false,
            total_works: 0,
            queue: Queue::default(),
        }
    }
}

impl AppState {
    pub fn add_work(&mut self, email: Email) {
        self.add_total_works();
        self.queue.add_queue(email);
        println!("{self:?}")
    }

    fn add_total_works(&mut self) {
        if !self.has_works {
            self.has_works = true;
        }
        self.total_works += 1
    }
}
