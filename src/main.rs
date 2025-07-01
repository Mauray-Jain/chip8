use std::{env, fs, io::Read};

use winit::event_loop::EventLoop;

use crate::app::App;

mod chip8;
mod renderer;
mod app;
mod time;

fn main() {
    env_logger::init();

    let mut args = env::args();
    args.next();

    let Some(path) = args.next() else {
        eprintln!("Need a ROM path");
        return;
    };

    let mut file = fs::File::open(path).expect("Unable to open file");
    let mut content = vec![];
    file.read_to_end(&mut content).expect("Unable to read file");
    drop(file);

    let event_loop = EventLoop::with_user_event().build().unwrap();
    let mut app = App::new([0.0, 0.25, 0.0], content);
    event_loop.run_app(&mut app).unwrap();
}
