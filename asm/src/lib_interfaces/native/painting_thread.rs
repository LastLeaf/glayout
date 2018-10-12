use std::thread;
use std::sync::{Arc, Mutex};
use std::cell::Cell;
use super::gl;
use super::super::super::utils::PretendSend;

pub enum PaintingCommand {
    CustomCommand(Box<Fn(&mut gl::Gles2) -> () + Send>),
}

pub struct PaintingThread {
    thread_handle: thread::JoinHandle<()>,
    cmd_buffer: Arc<Mutex<Cell<Vec<PaintingCommand>>>>,
    cmd_buffer_pending: Cell<Vec<PaintingCommand>>,
}

fn exec_command(ctx: &mut gl::Gles2, cmd: PaintingCommand) {
    use self::PaintingCommand::*;
    match cmd {
        CustomCommand(f) => {
            f(ctx);
        }
    }
}

impl PaintingThread {
    pub fn new(ctx: gl::Gles2) -> Self {
        let cmd_buffer = Arc::new(Mutex::new(Cell::new(vec![])));
        let cmd_buffer_self = cmd_buffer.clone();
        let cmd_buffer_pending = Cell::new(vec![]);
        let thread_handle = thread::Builder::new()
            .spawn(move || {
                let mut ctx = PretendSend::new(ctx);
                loop {
                    thread::park();
                    let buf = cmd_buffer.lock().unwrap().replace(vec![]);
                    for cmd in buf {
                        exec_command(&mut ctx, cmd);
                    }
                }
            })
            .unwrap();
        Self {
            thread_handle,
            cmd_buffer: cmd_buffer_self,
            cmd_buffer_pending,
        }
    }

    pub fn append_command(&mut self, cmd: PaintingCommand) {
        self.cmd_buffer_pending.get_mut().push(cmd);
    }

    pub fn redraw(&mut self) {
        let pending = self.cmd_buffer_pending.replace(vec![]);
        self.cmd_buffer.lock().unwrap().set(pending);
        self.thread_handle.thread().unpark();
    }
}
