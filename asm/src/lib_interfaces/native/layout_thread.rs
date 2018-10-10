use std::cmp::{Ord, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::binary_heap::BinaryHeap;
use std::time::SystemTime;
use glutin;
use super::super::utils::PretendSend;

lazy_static! {
    static ref LAYOUT_THREAD: Arc<Mutex<LayoutThread>> = Arc::new(Mutex::new(LayoutThread::new()));
}

pub enum EventDetail {
    WindowEvent(glutin::WindowEvent),
    TimeoutEvent,
    AnimationFrameEvent,
}

struct Event {
    event_id: usize,
    time: SystemTime,
    detail: EventDetail,
    callback: PretendSend<Box<Fn(EventDetail)>>,
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time && self.event_id == other.event_id
    }
}
impl Eq for Event {}
impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.time > other.time { Ordering::Less }
        else if self.time < other.time { Ordering::Greater }
        else if self.event_id > other.event_id { Ordering::Less }
        else if self.event_id < other.event_id { Ordering::Greater }
        else { Ordering::Equal }
    }
}

impl Event {
    fn dispatch(self) {
        let callback = self.callback;
        let detail = self.detail;
        (*callback)(detail)
    }
}

struct LayoutThread {
    thread_handle: thread::JoinHandle<()>,
    events_queue: Arc<Mutex<BinaryHeap<Event>>>,
    event_id_inc: usize,
}

impl LayoutThread {
    fn new() -> Self {
        let events_queue = Arc::new(Mutex::new(BinaryHeap::new()));
        let events_queue_self = events_queue.clone();
        let thread_handle = thread::Builder::new()
            .spawn(move || {
                loop {
                    thread::park();
                    loop {
                        let ev_option = {
                            let mut q_mutex = events_queue.lock().unwrap();
                            let q: &mut BinaryHeap<Event> = &mut *q_mutex;
                            if q.is_empty() {
                                None
                            } else {
                                if q.peek().unwrap().time > SystemTime::now() {
                                    None
                                } else {
                                    Some(q.pop().unwrap())
                                }
                            }
                        };
                        match ev_option {
                            None => { break },
                            Some(ev) => {
                                ev.dispatch();
                            }
                        }
                    }
                }
            })
            .unwrap();
        Self {
            thread_handle,
            events_queue: events_queue_self,
            event_id_inc: 0,
        }
    }

    fn push_event<F: 'static>(&mut self, time: SystemTime, detail: EventDetail, callback: F) where F: Fn(EventDetail) {
        let mut q = self.events_queue.lock().unwrap();
        if q.is_empty() {
            self.event_id_inc = 0;
        }
        let event_id = self.event_id_inc;
        self.event_id_inc += 1;
        q.push(Event {
            event_id,
            time,
            detail,
            callback: PretendSend::new(Box::new(callback)),
        })
    }
}

pub fn init() {
    // ensure thread is created
    LAYOUT_THREAD.lock().unwrap().event_id_inc = 0;
}

pub fn push_event<F: 'static>(time: SystemTime, detail: EventDetail, callback: F) where F: Fn(EventDetail) {
    LAYOUT_THREAD.lock().unwrap().push_event(time, detail, callback);
}

pub fn wakeup() {
    LAYOUT_THREAD.lock().unwrap().thread_handle.thread().unpark();
}
