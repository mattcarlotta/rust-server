use super::Worker;
use std::sync::{mpsc, Arc, Mutex};

pub type Task = Box<dyn FnOnce() + Send + 'static>;

pub enum Signal {
    NewTask(Task),
    Terminate,
}

pub struct TaskPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Signal>,
}

impl TaskPool {
    /// Create a new TaskPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> Result<Self, &'static str> {
        if size <= 0 {
            return Err("You must specify the maximum number of threads");
        }

        let (sender, receiver) = mpsc::channel();

        // Arc lets multiple workers own the receiver, and Mutex ensures one worker gets a job from the receiver at a time.
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        Ok(TaskPool { workers, sender })
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let task = Box::new(f);

        self.sender.send(Signal::NewTask(task)).unwrap();
    }
}

// allow workers to be gracefully closed on process termination
impl Drop for TaskPool {
    fn drop(&mut self) {
        // send signal to workers to terminate their task
        for _ in &self.workers {
            self.sender.send(Signal::Terminate).unwrap();
        }

        // take ownership of tasks from worker and join them
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
