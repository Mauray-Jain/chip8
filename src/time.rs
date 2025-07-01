use std::time::Instant;

pub struct Timer {
    start: Instant,
    prev: f32,
    pub acc: f32,
    delta: f32,
}

impl Timer {
    pub fn new() -> Self {
        Self { start: Instant::now(), prev: 0.0, acc: 0.0, delta: 0.0 }
    }

    pub fn update(&mut self) {
        let now = self.start.elapsed().as_secs_f32();
        self.delta = now - self.prev;
        self.acc += self.delta;
        self.prev = now;
    }

    pub fn reset(&mut self) {
        self.start = Instant::now();
        self.prev = 0.0;
        self.delta = 0.0;
        self.acc = 0.0;
    }
}
