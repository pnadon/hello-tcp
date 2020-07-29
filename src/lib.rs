use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::sync::mpsc;

enum Message {
    NewJob(Job),
    Terminate,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    /// Creates a new ThreadPool
    /// 
    /// size: number of threads in the pool
    /// 
    /// # Panics
    /// 
    /// Panics if the size is 0
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender}
    }

    /// Sends the task `f` through a channel to an awaiting Worker
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
        println!("Broadcasting terminate message");

        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        for worker in &mut self.workers {
            println!("Worker {} is being shut down", worker.id);
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
        let thread = thread::spawn(move || loop {
            match receiver.lock().unwrap().recv() {
                Ok(Message::NewJob(job)) => {
                    println!("Worker {} received job, executing...", id);
                    job();
                    println!("Worker {} completed job.", id);
                },
                Ok(Message::Terminate) => {
                    println!("Worker {} received termination signal.", id);
                    break;
                },
                Err(_) => panic!(),
            }
        });

        Worker { id, thread: Some(thread)}
    }
}