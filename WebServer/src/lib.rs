pub mod thread_pool {
    use std::sync::mpsc::Receiver;
    use std::sync::{Arc, mpsc, Mutex};
    use std::thread;

    type Job = Box<dyn FnOnce() + Send + 'static>;

    enum Message {
        NewJob(Job),
        Terminate,
    }


    pub struct ThreadPool {
        workers: Vec<Worker>,
        sender: mpsc::Sender<Message>,
    }

    impl ThreadPool {
        pub fn new(size: usize) -> ThreadPool {
            assert!(size > 0);
            let mut workers = Vec::with_capacity(size);
            let (sender, receiver) = mpsc::channel();
            let receiver = Arc::new(Mutex::new(receiver));
            for id in 0..size {
                workers.push(Worker::new(id, Arc::clone(&receiver)))
            }
            ThreadPool {
                workers,
                sender,
            }
        }
        pub fn execute<F>(&self, f: F)
            where
                F: FnOnce() + Send + 'static {
            let job = Box::new(f);
            self.sender.send(Message::NewJob(job)).unwrap()
        }
    }

    impl Drop for ThreadPool {
        fn drop(&mut self) {
            for _ in &self.workers {
                self.sender.send(Message::Terminate);
            }
            for worker in &mut self.workers {
                println!("shutting down woker:{}", worker.id);
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
        fn new(id: usize, receiver: Arc<Mutex<Receiver<Message>>>) -> Worker {
            let thread = thread::spawn(move || loop {
                let job = receiver
                    .lock()
                    .unwrap()
                    .recv()
                    .unwrap();

                match job {
                    Message::NewJob(job) => {
                        println!("Worker {} got a job; executing.", id);
                        job()
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
}