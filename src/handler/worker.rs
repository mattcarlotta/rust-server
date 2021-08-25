use super::Signal;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub struct Worker {
    pub id: usize,
    pub thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Create a new worker.
    ///
    /// The worker handles the executing of tasks on a receiving channel.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the sender is poisened.
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Signal>>>) -> Self {
        let thread = thread::spawn(move || loop {
            // call lock on the receiver to acquire the mutex,
            // and then we call unwrap to panic on any errors
            let signal = receiver.lock().unwrap().recv().unwrap();

            match signal {
                Signal::NewTask(task) => {
                    println!("Worker {} got a job; executing.", id);

                    task();
                }
                Signal::Terminate => {
                    println!("Worker {} was told to terminate.", id);

                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}
