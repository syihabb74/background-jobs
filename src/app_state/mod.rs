#[derive(Debug)]
pub struct AppState {
    pub has_task: bool,
    pub total_task: u32,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            has_task: false,
            total_task: 0,
        }
    }

    pub fn increase_task(&mut self) {
        if self.total_task == 0 {
            self.has_task = true
        }
        self.total_task += 1
    }

    pub fn decrease_task(&mut self) {
        if self.total_task != 0 {
            self.total_task -= 1;
        }
        if self.total_task == 0 {
            self.has_task = false;
        }
    }

}
