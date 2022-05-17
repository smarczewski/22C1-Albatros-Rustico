// ThreadPool as implemented on The Rust Programming Language Book Chapter 20

use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

// A thread pool is a group of spawned threads that are waiting and ready to handle a task.
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

// A type alias for a trait object that holds the type of closure that execute receives
type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        // modify to handle errors accordingly
        // pub fn new(size: usize) -> Result<ThreadPool, PoolCreationError> {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            // create some workers and store them in vector
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        for worker in &mut self.workers {
            println!("Shutting down worker: {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

// This data structure is a Worker, which is a common term in pooling implementations.
// Think of people working in the kitchen at a restaurant: the workers wait until orders
// come in from customers, and then they’re responsible for taking those orders and filling them.
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            // the closure loops forever,
            // asking the receiving end of the channel for a job and
            // running the job when it gets one
            let message = receiver.lock().unwrap().recv().unwrap();
            // We first call lock on the receiver to acquire the mutex, and then
            // we call unwrap to panic on any errors. Acquiring a lock might fail if
            // the mutex is in a poisoned state, which can happen if some other thread
            // panicked while holding the lock rather than releasing the lock.
            // If we get the lock on the mutex, we call recv to receive a Job from the channel.
            // A final unwrap moves past any errors here as well, which might occur
            // if the thread holding the sending side of the channel has shut down

            match message {
                Message::NewJob(job) => {
                    job();
                }
                Message::Terminate => {
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
