use std::{
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
};

pub enum Method {
    Post,
    Get,
}
pub struct HttpRequest {
    pub method: Method,
    pub url: String,
}
impl HttpRequest {
    pub fn new(line: String) -> HttpRequest {
        let m = if line.starts_with("GET") {
            Method::Get
        } else {
            Method::Post
        };
        let items = line.split(" ");
        let mut url = String::from(".");
        for item in items {
            if item.starts_with("/") {
                if item == "/" {
                    url = String::from("index.html");
                } else {
                    url.push_str(item);
                }
            }
        }
        HttpRequest {
            method: m,
            url: url,
        }
    }
}

pub enum Message {
    NewJob(Job),
    Terminate,
}
pub struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}
type Job = Box<dyn FnOnce() + Send + 'static>;
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Message>>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let msg = receiver.lock().unwrap().recv();
            match msg {
                Ok(message) => match message {
                    Message::NewJob(job) => {
                        println!("Work {id} 获得一个任务;开始执行...");
                        job();
                    }
                    Message::Terminate => {
                        println!("Work {id} 可以下班了...");
                        break;
                    }
                },
                Err(_) => {
                    println!("Work {id} 失去通讯,下班了...");
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
impl ThreadPool {
    pub fn new(count: usize) -> ThreadPool {
        assert!(count > 0);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(count);
        for i in 0..count {
            workers.push(Worker::new(i, Arc::clone(&receiver)));
        }
        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        match self.sender.as_ref().unwrap().send(Message::NewJob(job)) {
            Ok(_) => println!("任务已分配！"),
            Err(_) => println!("分配任务失败！"),
        };
    }
}
impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("通知 Works 下班...");
        for _ in &mut self.workers {
            self.sender
                .as_ref()
                .unwrap()
                .send(Message::Terminate)
                .unwrap();
        }
        for w in &mut self.workers {
            println!("Work {} 下班了！", w.id);
            if let Some(thread) = w.thread.take() {
               thread.join().unwrap();
            }
        }
        drop(self.sender.take());
    }
}
