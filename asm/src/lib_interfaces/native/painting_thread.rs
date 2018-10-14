use std::thread;
use std::sync::{Arc, Mutex};
use std::cell::Cell;
use super::{gl, tex_manager};
use super::gl::Gl as Gl;
use super::super::super::utils::PretendSend;

pub enum PaintingCommand {
    CustomCommand(Box<Fn(&mut Gl, &mut super::tex_manager::TexManager) -> () + Send>),
}

pub struct PaintingThread {
    tex_size: i32,
    tex_count: i32,
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
    pub fn new<F>(ctx: Box<Gl>, thread_init: F) -> Self where F: Fn() -> () + Send + 'static {
        let max_tex_size = unsafe {
            let mut ret = 4096;
            ctx.GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut ret as *mut i32);
            ret
        };
        let max_tex_count = unsafe {
            let mut ret = 16;
            ctx.GetIntegerv(gl::MAX_TEXTURE_IMAGE_UNITS, &mut ret as *mut i32);
            ret
        };

        let cmd_buffer = Arc::new(Mutex::new(Cell::new(vec![])));
        let cmd_buffer_self = cmd_buffer.clone();
        let cmd_buffer_pending = Cell::new(vec![]);
        let thread_handle = thread::Builder::new()
            .spawn(move || {
                thread_init();
                let mut ctx = PretendSend::new(ctx);
                let mut tex_manager = Box::new(tex_manager::TexManager::new(&mut ctx, max_tex_size, max_tex_count));
                loop {
                    thread::park();
                    let buf = cmd_buffer.lock().unwrap().replace(vec![]);
                    for cmd in buf {
                        exec_command(&mut ctx, &mut tex_manager, cmd);
                    }
                }
            })
            .unwrap();
        Self {
            tex_size: max_tex_size,
            tex_count: max_tex_count,
            thread_handle,
            cmd_buffer: cmd_buffer_self,
            cmd_buffer_pending,
        }
    }

    pub fn get_tex_size(&self) -> i32 {
        self.tex_size
    }

    pub fn get_tex_count(&self) -> i32 {
        self.tex_count
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
