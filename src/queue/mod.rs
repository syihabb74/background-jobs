use std::{collections::VecDeque};

use crate::{email::{Email}};




    #[derive(Debug)]
    pub struct Queue {
        pub queue : VecDeque<Email>,
       
    }

    impl Queue {
        pub fn default () -> Self {
            Self { queue: VecDeque::new() }
        }

        pub fn add_queue (&mut self, email : Email) {
            
            self.queue.push_back(email);

        }

        pub fn remove_queue(&mut self) {
            let email = self.queue.pop_back();
            if let Some(email_exist) = email {
                // email_exist . sending
            }
        }

    }

