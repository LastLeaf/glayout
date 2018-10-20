use std::cmp::{Ord, Ordering};
use std::sync::{Arc, Mutex, Barrier};
use std::time::Duration;
use std::thread;
use std::collections::binary_heap::BinaryHeap;
use std::time::{Instant, SystemTime};
use glutin;
use super::super::utils::PretendSend;

lazy_static! {
    static ref LAYOUT_THREAD: Arc<Mutex<LayoutThread>> = Arc::new(Mutex::new(LayoutThread::new()));
    static ref UI_THREAD_TASK: Arc<Mutex<Option<Box<Fn(&glutin::EventsLoop) -> () + Send>>>> = Arc::new(Mutex::new(None));
}

const ANIMATION_FRAME_INTERVAL: u32 = 16_666_666;

#[derive(Debug)]
pub enum EventDetail {
    WindowEvent(glutin::WindowEvent, i32),
    TimeoutEvent,
    AnimationFrameEvent,
    ImageLoadEvent,
}

struct Event {
    event_id: usize,
    time: Instant,
    detail: EventDetail,
    callback: PretendSend<Box<Fn(Instant, EventDetail)>>,
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
        (*callback)(self.time, detail)
    }
}

struct LayoutThread {
    thread_handle: thread::JoinHandle<()>,
    events_queue: Arc<Mutex<BinaryHeap<Event>>>,
    event_id_inc: usize,
    ui_thread_handle: Option<glutin::EventsLoopProxy>,
    animation_frame_enabled: bool,
    animation_frame_scheduled: bool,
}

impl LayoutThread {
    fn new() -> Self {
        let events_queue = Arc::new(Mutex::new(BinaryHeap::new()));
        let events_queue_self = events_queue.clone();
        let thread_handle = thread::Builder::new()
            .spawn(move || {
                thread::park();
                loop {
                    let mut resume_time = None;
                    loop {
                        let ev_option = {
                            let mut q_mutex = events_queue.lock().unwrap();
                            let q: &mut BinaryHeap<Event> = &mut *q_mutex;
                            if q.is_empty() {
                                None
                            } else {
                                let peek_time = q.peek().unwrap().time;
                                let now = Instant::now();
                                if peek_time > now {
                                    resume_time = Some(peek_time.duration_since(now));
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
                    match resume_time {
                        Some(t) => {
                            thread::park_timeout(t);
                        },
                        None => {
                            thread::park();
                        },
                    }
                }
            })
            .unwrap();
        Self {
            thread_handle,
            events_queue: events_queue_self,
            event_id_inc: 0,
            ui_thread_handle: None,
            animation_frame_enabled: false,
            animation_frame_scheduled: false,
        }
    }

    fn push_event<F: 'static>(&mut self, time: Instant, detail: EventDetail, callback: Box<F>, thread_id: thread::ThreadId) where F: Fn(Instant, EventDetail) {
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
            callback: PretendSend::new_with_thread_id(callback, thread_id),
        })
    }
}

pub fn init() {
    // ensure thread is created
    LAYOUT_THREAD.lock().unwrap().event_id_inc = 0;
}

pub fn push_event_from_layout_thread<F: 'static>(time: Instant, detail: EventDetail, callback: F) where F: Fn(Instant, EventDetail) {
    let thread_id = thread::current().id();
    if thread_id != LAYOUT_THREAD.lock().unwrap().thread_handle.thread().id() {
        panic!("push_event_from_layout_thread can only be called in layout thread");
    }
    LAYOUT_THREAD.lock().unwrap().push_event(time, detail, Box::new(callback), thread_id);
}

pub fn push_event<F: 'static + Send>(time: Instant, detail: EventDetail, callback: F) where F: Fn(Instant, EventDetail) {
    let thread_id = LAYOUT_THREAD.lock().unwrap().thread_handle.thread().id();
    LAYOUT_THREAD.lock().unwrap().push_event(time, detail, Box::new(callback), thread_id);
}

pub fn wakeup() {
    LAYOUT_THREAD.lock().unwrap().thread_handle.thread().unpark();
}

pub fn set_ui_thread_handle(h: glutin::EventsLoopProxy) {
    LAYOUT_THREAD.lock().unwrap().ui_thread_handle = Some(h);
}

pub fn exec_ui_thread_task(events_loop: &glutin::EventsLoop) {
    let mut f = UI_THREAD_TASK.lock().unwrap();
    let f = f.take().unwrap();
    (*f)(events_loop);
}

pub fn exec_in_ui_thread(f: Box<Fn(&glutin::EventsLoop) -> () + Send>) {
    let barrier = Arc::new(Barrier::new(2));
    let barrier_self = barrier.clone();
    {
        let mut task = UI_THREAD_TASK.lock().unwrap();
        if (*task).is_some() { panic!() };
        *task = Some(Box::new(move |events_loop| {
            f(events_loop);
            barrier.wait();
        }));
    }
    {
        let lt = LAYOUT_THREAD.lock().unwrap();
        lt.ui_thread_handle.as_ref().unwrap().wakeup().unwrap();
    }
    barrier_self.wait();
}

fn schedule_animation_frame(layout_thread: &mut LayoutThread, time: Instant) {
    layout_thread.push_event(time, EventDetail::AnimationFrameEvent, Box::new(move |_time: Instant, _detail| {
        let next_frame_time = Instant::now() + Duration::new(0, ANIMATION_FRAME_INTERVAL);
        {
            let mut layout_thread = LAYOUT_THREAD.lock().unwrap();
            if !layout_thread.animation_frame_enabled {
                layout_thread.animation_frame_scheduled = false;
                return;
            }
        }
        let dur = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
        let secs = dur.as_secs() as f64;
        let nanos = dur.subsec_nanos() as f64;
        super::super::animation_frame(secs * 1000. + nanos / 1000_000.);
        super::trigger_painting();
        {
            let mut layout_thread = LAYOUT_THREAD.lock().unwrap();
            schedule_animation_frame(&mut layout_thread, next_frame_time);
        }
    }), thread::current().id());
}

pub fn set_animation_frame_enabled(enabled: bool) {
    let mut layout_thread = LAYOUT_THREAD.lock().unwrap();
    if layout_thread.animation_frame_enabled == enabled {
        return
    }
    layout_thread.animation_frame_enabled = enabled;
    if enabled && !layout_thread.animation_frame_scheduled {
        layout_thread.animation_frame_scheduled = true;
        schedule_animation_frame(&mut layout_thread, Instant::now() + Duration::new(0, ANIMATION_FRAME_INTERVAL));
        layout_thread.thread_handle.thread().unpark();
    }
}
