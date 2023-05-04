use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

pub struct ThreadPool {
    // ThreadPool owns the workers vector and the sending side of the channel
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

// Create a job type that is a trait object that holds the type of closure
type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    // NewJob holds a Job
    NewJob(Job),
    Terminate, // Terminate variant
}

impl ThreadPool {
    // Create a new ThreadPool
    // The size is the number of threads in the pool
    // The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        // create a channel and store the sending end in the ThreadPool
        let (sender, receiver) = mpsc::channel();
        // wrap the receiving end in a Mutex and an Arc
        let receiver = Arc::new(Mutex::new(receiver));
        // create a vector to hold the threads
        let mut workers = Vec::with_capacity(size);
        // iterate over the range of ids
        for id in 0..size {
            // create some threads and store them in the vector
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }
    // The execute method takes a closure of type F that takes no parameters and returns nothing
    pub fn execute<F>(&self, f: F)
        where
            F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);

        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");
        // iterate over the workers vector
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all workers.");
        // iterate over the workers vector
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        // create a new thread
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();

            println!("Worker {} got a job; executing.", id);

            match message {
                Message::NewJob(job) => {
                    println!("Worker {} got a job; executing.", id);
                    job();
                }
                Message::Terminate => {
                    println!("Worker {} was told to terminate.", id);
                    break;
                }
            }
        });
        // return a new Worker
        Worker { id, thread: Some(thread) }
    }
}