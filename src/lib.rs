use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    // keyboard::PhysicalKey,
    window::Window,
};

mod renderer;
use crate::renderer::QuadRenderer;
use crate::renderer::Rect;

const QUADS: &[Rect] = &[
    Rect { x: 0.0, y: 0.0, w: 0.25, h: 0.25 },
    Rect { x: -0.5, y: -0.5, w: 0.25, h: 0.25 },
    Rect { x: 0.5, y: 0.5, w: 0.25, h: 0.25 },
];

pub struct App {
    state: Option<QuadRenderer>,
}

impl App {
    pub fn new() -> Self {
        Self { state: None }
    }
}

impl ApplicationHandler<QuadRenderer> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes();
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.state = Some(pollster::block_on(QuadRenderer::new(window)));
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: QuadRenderer) {
        self.state = Some(event);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = &mut self.state else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => match state.render_quads(QUADS, [0.0, 0.15, 0.0]) {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                    let size = state.window.inner_size();
                    state.resize(size.width, size.height);
                }
                Err(e) => {
                    log::error!("Unable to render: {}", e);
                }
            },
            // WindowEvent::KeyboardInput {
            //     event:
            //         KeyEvent {
            //             physical_key: PhysicalKey::Code(code),
            //             state: key_state,
            //             ..
            //         },
            //     ..
            // } => state.handle_key(event_loop, code, key_state.is_pressed()),
            _ => {}
        }
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = App::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}
