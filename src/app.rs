use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use crate::renderer::Rect;
use crate::time::Timer;
use crate::{
    chip8::{self, Chip8},
    renderer::QuadRenderer,
};

pub struct App {
    state: Option<QuadRenderer>,
    chip8: Chip8,
    quads: Vec<Rect>,
    color: [f32; 3],
    timer: Timer,
}

impl App {
    pub fn new(color: [f32; 3], content: Vec<u8>) -> Self {
        Self {
            state: None,
            chip8: Chip8::new(content),
            quads: vec![],
            color,
            timer: Timer::new(),
        }
    }

    pub fn update_keypad(&mut self, code: KeyCode, pressed: bool) {
        let code = match code {
            KeyCode::KeyX => Some(0x0),
            KeyCode::Digit1 => Some(0x1),
            KeyCode::Digit2 => Some(0x2),
            KeyCode::Digit3 => Some(0x3),
            KeyCode::KeyQ => Some(0x4),
            KeyCode::KeyW => Some(0x5),
            KeyCode::KeyE => Some(0x6),
            KeyCode::KeyA => Some(0x7),
            KeyCode::KeyS => Some(0x8),
            KeyCode::KeyD => Some(0x9),
            KeyCode::KeyZ => Some(0xa),
            KeyCode::KeyC => Some(0xb),
            KeyCode::Digit4 => Some(0xc),
            KeyCode::KeyR => Some(0xd),
            KeyCode::KeyF => Some(0xe),
            KeyCode::KeyV => Some(0xf),
            _ => None,
        };

        let Some(code) = code else {
            return;
        };

        self.chip8.keypad[code] = pressed;
    }

    pub fn update_quads(&mut self) {
        let Some(state) = &self.state else {
            return;
        };

        let size = state.window.inner_size();
        let mut quads = vec![];
        let chip8_width = chip8::CHIP8_WIDTH as f32;
        let chip8_height = chip8::CHIP8_HEIGHT as f32;
        let w = (1.0 / size.width as f32) * chip8_width;
        let h = (1.0 / size.height as f32) * chip8_height;

        for i in 0..chip8::CHIP8_WIDTH {
            for j in 0..chip8::CHIP8_HEIGHT {
                if self.chip8.screen[j * chip8::CHIP8_WIDTH + i] != 0 {
                    quads.push(Rect {
                        x: (i as f32) / (chip8_width / 2.0) - 1.0,
                        y: 1.0 - (j as f32) / (chip8_height / 2.0),
                        w,
                        h,
                    });
                }
            }
        }

        self.quads = quads;
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
        if let None = &self.state {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                self.update_quads();
                self.state.as_mut().unwrap().resize(size.width, size.height);
            }
            WindowEvent::RedrawRequested => {
                self.timer.update();
                self.chip8.tick(&mut self.timer);

                if self.chip8.draw_flag {
                    self.update_quads();
                    self.chip8.draw_flag = false;
                }

                let state = self.state.as_mut().unwrap();

                match state.render_quads(self.quads.as_slice(), self.color) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let size = state.window.inner_size();
                        state.resize(size.width, size.height);
                    }
                    Err(e) => {
                        log::error!("Unable to render: {}", e);
                    }
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => self.update_keypad(code, key_state.is_pressed()),
            _ => {}
        }
    }
}
