use std::sync::{Condvar, Mutex};

pub struct CountDownLatch((Mutex<usize>, Condvar));

impl CountDownLatch {
    pub fn new(count: usize) -> Self {
        CountDownLatch((Mutex::new(count), Condvar::new()))
    }

    pub fn wait(&self) {
        let (lock, cvar) = &self.0;
        let mut count = lock.lock().unwrap();
        while *count > 0 {
            count = cvar.wait(count).unwrap();
        }
    }

    pub fn count_down(&self) {
        let (lock, cvar) = &self.0;
        let mut count = lock.lock().unwrap();
        *count -= 1;
        if *count == 0 {
            cvar.notify_all();
        }
    }
}
