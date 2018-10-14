use std::thread;
use std::sync::{Arc, Mutex, mpsc, Barrier};
use std::cell::Cell;
use super::{gl, tex_manager};
use super::gl::Gles2 as Gl;
use super::super::super::utils::PretendSend;

enum PaintingJob {
    Immediate(Box<Fn(&mut Gl, &mut super::tex_manager::TexManager) -> () + Send>),
    Queue,
}

pub enum PaintingCommand {
    CustomCommand(Box<Fn(&mut Gl, &mut super::tex_manager::TexManager) -> () + Send>),
}

pub struct PaintingThread {
    tex_size: Arc<Mutex<Cell<i32>>>,
    tex_count: Arc<Mutex<Cell<i32>>>,
    sender: mpsc::Sender<PaintingJob>,
    thread_handle: thread::JoinHandle<()>,
    cmd_buffer: Arc<Mutex<Cell<Vec<PaintingCommand>>>>,
    cmd_buffer_pending: Cell<Vec<PaintingCommand>>,
}

fn exec_command(ctx: &mut Gl, tex_manager: &mut super::tex_manager::TexManager, cmd: PaintingCommand) {
    use self::PaintingCommand::*;
    match cmd {
        CustomCommand(f) => {
            f(ctx, tex_manager);
        }
    }
}

impl PaintingThread {
    pub fn new<F>(thread_init: F, ready_barrier: Arc<Barrier>) -> Self where F: Fn() -> Box<Gl> + Send + 'static {
        let cmd_buffer = Arc::new(Mutex::new(Cell::new(vec![])));
        let cmd_buffer_self = cmd_buffer.clone();
        let cmd_buffer_pending = Cell::new(vec![]);
        let max_tex_size = Arc::new(Mutex::new(Cell::new(0)));
        let max_tex_count = Arc::new(Mutex::new(Cell::new(0)));
        let tex_size = max_tex_size.clone();
        let tex_count = max_tex_size.clone();
        let (sender, receiver) = mpsc::channel();
        let thread_handle = thread::Builder::new()
            .spawn(move || {
                let ctx = thread_init();

                let tex_size = unsafe {
                    let mut ret = 4096;
                    ctx.GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut ret as *mut i32);
                    ret
                };
                max_tex_size.lock().unwrap().set(tex_size);
                let tex_count = unsafe {
                    let mut ret = 16;
                    ctx.GetIntegerv(gl::MAX_TEXTURE_IMAGE_UNITS, &mut ret as *mut i32);
                    ret
                };
                max_tex_count.lock().unwrap().set(tex_count);

                let mut ctx = PretendSend::new(ctx);
                let mut tex_manager = Box::new(tex_manager::TexManager::new(&mut ctx, tex_size, tex_count));
                ready_barrier.wait();
                loop {
                    match receiver.recv().unwrap() {
                        PaintingJob::Immediate(f) => {
                            f(&mut ctx, &mut tex_manager);
                        },
                        PaintingJob::Queue => {
                            let buf = cmd_buffer.lock().unwrap().replace(vec![]);
                            for cmd in buf {
                                exec_command(&mut ctx, &mut tex_manager, cmd);
                            }
                        }
                    }
                }
            })
            .unwrap();
        Self {
            tex_size,
            tex_count,
            sender,
            thread_handle,
            cmd_buffer: cmd_buffer_self,
            cmd_buffer_pending,
        }
    }

    pub fn get_tex_size(&self) -> i32 {
        self.tex_size.lock().unwrap().get()
    }

    pub fn get_tex_count(&self) -> i32 {
        self.tex_count.lock().unwrap().get()
    }

    pub fn exec(&mut self, f: Box<Fn(&mut Gl, &mut super::tex_manager::TexManager) -> () + Send>) {
        self.sender.send(PaintingJob::Immediate(f)).unwrap();
    }

    pub fn append_command(&mut self, cmd: PaintingCommand) {
        self.cmd_buffer_pending.get_mut().push(cmd);
    }

    pub fn redraw(&mut self) {
        // FIXME consider swap but not create new vec
        let pending = self.cmd_buffer_pending.replace(vec![]);
        self.cmd_buffer.lock().unwrap().set(pending);
        self.sender.send(PaintingJob::Queue).unwrap();
    }
}
