use std::sync::{Arc, Mutex, Condvar};
use std::collections::VecDeque;
//use crate::video_4::Cell;

//channel flavors-
// synchronous, asynchronous(unbounded, send doesnt block), rendezvous , oneshot 

//TODO:
//1. implement rendezvous channel
//2. check asynchronous impl, why does send not block
//      - use LinkedList instead of  VecDeq
//3. synchronous- how to implement bounded 
//      -> atomic vecdeque with thread::park, thread::notify
//4. mpsc, crossbeam, flume
struct Inner<T>{
    queue : VecDeque<T>,
    sender_size : usize
}

impl <T> Default for Inner<T>{
    fn default() -> Self {
        Self { queue: VecDeque::default(), sender_size: 0 }
    }
}
struct Shared <T> {
    qu : Mutex<Inner<T>>,
    available : Condvar
}

pub struct Sender <T> {
    inner : Arc<Shared<T>>    
}

pub struct Receiver <T>{
    inner : Arc<Shared<T>>,
    buffer : VecDeque<T>
}


impl <T> Sender<T>{
    pub fn send(&self, t: T) {
        let mut inner = self.inner.qu.lock().unwrap();
        inner.queue.push_back(t);
        // notify_one on condvar notifies receiver ,that then can take a mutex, hence, pre-emptively dropping mutex before calling notify
        drop(inner);
        self.inner.available.notify_one();
    }
}

// we are not using #derive[Clone] here because: impl <T : Clone> for Sender<T> ; compiler enforces Clone on T, since our inner is Arc, we dont need to enforce Clone on T
// using inner.clone is not correct. (.) will recursively deref to T for which clone might be implemented, since arc points to struct with t

impl <T> Clone for Sender<T>{
    fn clone(&self) -> Self{
        let mut inner  = self.inner.qu.lock().unwrap();
        inner.sender_size += 1;
        drop(inner);
        Self { inner: Arc::clone(&self.inner) } 
    }
}

impl <T> Drop for Sender<T> {
    fn drop(&mut self) {
        let mut inner = self.inner.qu.lock().unwrap();
        inner.sender_size = dbg!(inner.sender_size) - 1;
        //drop(inner);
        if inner.sender_size == 0{
            self.inner.available.notify_one();
        }
    }
}


impl <T> Receiver<T>{
    pub fn recv(&mut self) -> Option<T>{
        if !self.buffer.is_empty(){
            return self.buffer.pop_front()
        }
        let mut inner = self.inner.qu.lock().unwrap();
        loop{
            match inner.queue.pop_front(){
                None if inner.sender_size ==0 => return None,
                None => {
                    // wait consumes mutex guard and releases the lock and thread goes to sleep
                    // thread supposedly awakens on available Condvar notification and gets mutex lock back, hence stored in inner
                    inner = self.inner.available.wait(inner).unwrap();
                },
                Some(v) => {
                    std::mem::swap(&mut self.buffer, &mut inner.queue);
                    return Some(v)
                }
            }
        }
    }
}

pub fn channel<T> () -> (Sender<T>, Receiver<T>){
    let inner_inner = Inner {queue : VecDeque::default(), sender_size : dbg!(1)};
    let inner = Arc::new(Shared { qu : Mutex::new(inner_inner), available : Condvar::new()});
    (Sender  {inner : inner.clone()}, Receiver { inner : inner.clone(), buffer : VecDeque::default()})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ping_pong(){
        let (tx, mut rx) = channel();
        tx.send("hello there");
        assert_eq!(Some("hello there"), rx.recv());
        {
            let t = tx;
        }
        assert_eq!(None, rx.recv());
//        println!("message: {:?}", rx.recv());
    }

    #[test]
    fn multiple(){
        let (tx, mut rx) = channel();
        tx.send("hello");
        tx.send("there");
        tx.send("what is my help");
        assert_eq!(Some("hello"), rx.recv());
        {
            let _t = tx;
        }
        assert_eq!(Some("there"), rx.recv());
        assert_eq!(Some("what is my help"), rx.recv());
        assert_eq!(None, rx.recv());
    }
}

fn main(){

}