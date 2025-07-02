use rand::Rng;

use crate::time::Timer;

pub const CHIP8_WIDTH: usize = 64;
pub const CHIP8_HEIGHT: usize = 32;
pub const CHIP8_MEM: usize = 4096;
pub const ONE_BY_CLOCK_SPEED: f32 = 1.0 / 500.0;
pub const ONE_BY_FPS: f32 = 1.0 / 60.0;
pub const CHIP8_FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];
const OP_SIZE: usize = 2;

enum ProgramCounterState {
    Next,
    Skip,
    Jmp(usize),
}

#[derive(Debug)]
pub struct Chip8 {
    v: [u8; 16],
    i: usize,
    pc: usize,
    sp: usize,
    stack: [usize; 16],
    mem: [u8; CHIP8_MEM],
    pub screen: [[u8; CHIP8_WIDTH]; CHIP8_HEIGHT],
    pub draw_flag: bool,
    pub beep: bool,
    delay: u8,
    sound: u8,
    keypad: [bool; 16],
    last_key: Option<u8>,
    waiting_key: bool,
}

impl Chip8 {
    pub fn new(content: Vec<u8>) -> Self {
        let mut ram = [0; CHIP8_MEM];
        ram[..CHIP8_FONTSET.len()].copy_from_slice(&CHIP8_FONTSET[..]);
        ram[0x200..content.len() + 0x200].copy_from_slice(&content);

        Self {
            v: [0; 16],
            i: 0,
            pc: 0x200,
            sp: 0,
            stack: [0; 16],
            mem: ram,
            screen: [[0; CHIP8_WIDTH]; CHIP8_HEIGHT],
            delay: 0,
            sound: 0,
            keypad: [false; 16],
            last_key: None,
            waiting_key: false,
            draw_flag: false,
            beep: false,
        }
    }

    pub fn tick(&mut self, timer: &mut Timer, clock_timer: &mut Timer) {
        if clock_timer.acc >= ONE_BY_CLOCK_SPEED {
            let op = self.get_op();
            self.exec_op(op);
            clock_timer.reset();
        }

        if timer.acc >= ONE_BY_FPS {
            if self.sound > 0 {
                self.sound -= 1;
            }
            if self.delay > 0 {
                self.delay -= 1;
            }

            self.beep = self.sound > 0;
            timer.reset();
        }
    }

    pub fn update_keypad(&mut self, code: u8, pressed: bool) {
        if self.waiting_key && !pressed {
            self.last_key = Some(code);
        }

        self.keypad[code as usize] = pressed;
    }

    fn get_op(&self) -> u16 {
        (self.mem[self.pc] as u16) << 8 | (self.mem[self.pc + 1] as u16)
    }

    fn exec_op(&mut self, op: u16) {
        let first = (op >> 12) as u8;
        let second = ((op >> 8) & 0xf) as u8;
        let third = ((op >> 4) & 0xf) as u8;
        let fourth = (op & 0xf) as u8;

        let nnn = (op & 0xfff) as usize;
        let kk = (op & 0xff) as u8;
        let x = second as usize;
        let y = third as usize;
        let n = fourth as usize;

        let pc_state = match (first, second, third, fourth) {
            (0x0, 0x0, 0xe, 0x0) => self.clear_display(),
            (0x0, 0x0, 0xe, 0xe) => self.ret(),
            (0x1, _, _, _) => self.jmp(nnn),
            (0x2, _, _, _) => self.call(nnn),
            (0x3, _, _, _) => self.skip_kk_eq(x, kk),
            (0x4, _, _, _) => self.skip_kk_ne(x, kk),
            (0x5, _, _, 0x0) => self.skip_vy_eq(x, y),
            (0x6, _, _, _) => self.load_kk(x, kk),
            (0x7, _, _, _) => self.add_kk(x, kk),
            (0x8, _, _, 0x0) => self.load_vy(x, y),
            (0x8, _, _, 0x1) => self.or(x, y),
            (0x8, _, _, 0x2) => self.and(x, y),
            (0x8, _, _, 0x3) => self.xor(x, y),
            (0x8, _, _, 0x4) => self.add_vy(x, y),
            (0x8, _, _, 0x5) => self.sub(x, y),
            (0x8, _, _, 0x6) => self.shr(x, y),
            (0x8, _, _, 0x7) => self.subn(x, y),
            (0x8, _, _, 0xe) => self.shl(x, y),
            (0x9, _, _, 0x0) => self.skip_vy_ne(x, y),
            (0xa, _, _, _) => self.load_addr(nnn),
            (0xb, _, _, _) => self.jmp(nnn + self.v[0] as usize),
            (0xc, _, _, _) => self.rand(x, kk),
            (0xd, _, _, _) => self.draw(x, y, n),
            (0xe, _, 0x9, 0xe) => self.skip_key_eq(x),
            (0xe, _, 0xa, 0x1) => self.skip_key_ne(x),
            (0xf, _, 0x0, 0x7) => self.load_delay(x),
            (0xf, _, 0x0, 0xa) => self.load_key(x),
            (0xf, _, 0x1, 0x5) => self.load_vx_delay(x),
            (0xf, _, 0x1, 0x8) => self.load_vx_sound(x),
            (0xf, _, 0x1, 0xe) => self.add_i(x),
            (0xf, _, 0x2, 0x9) => self.load_sprite(x),
            (0xf, _, 0x3, 0x3) => self.load_bcd(x),
            (0xf, _, 0x5, 0x5) => self.store_v0_vx(x),
            (0xf, _, 0x6, 0x5) => self.load_v0_vx(x),
            _ => panic!("Invalid instruction: {op:#x}"),
        };

        match pc_state {
            ProgramCounterState::Next => self.pc += OP_SIZE,
            ProgramCounterState::Skip => self.pc += 2 * OP_SIZE,
            ProgramCounterState::Jmp(addr) => self.pc = addr,
        }
    }

    fn clear_display(&mut self) -> ProgramCounterState {
        self.screen = [[0; CHIP8_WIDTH]; CHIP8_HEIGHT];
        ProgramCounterState::Next
    }

    fn ret(&mut self) -> ProgramCounterState {
        self.sp -= 1;
        ProgramCounterState::Jmp(self.stack[self.sp])
    }

    fn jmp(&mut self, nnn: usize) -> ProgramCounterState {
        ProgramCounterState::Jmp(nnn)
    }

    fn call(&mut self, nnn: usize) -> ProgramCounterState {
        self.stack[self.sp] = self.pc + OP_SIZE; // jmp to next instruction
        self.sp += 1;
        ProgramCounterState::Jmp(nnn)
    }

    fn skip_kk_eq(&mut self, x: usize, kk: u8) -> ProgramCounterState {
        if self.v[x] == kk {
            return ProgramCounterState::Skip;
        }
        ProgramCounterState::Next
    }

    fn skip_kk_ne(&mut self, x: usize, kk: u8) -> ProgramCounterState {
        if self.v[x] != kk {
            return ProgramCounterState::Skip;
        }
        ProgramCounterState::Next
    }

    fn skip_vy_eq(&mut self, x: usize, y: usize) -> ProgramCounterState {
        if self.v[x] == self.v[y] {
            return ProgramCounterState::Skip;
        }
        ProgramCounterState::Next
    }

    fn load_kk(&mut self, x: usize, kk: u8) -> ProgramCounterState {
        self.v[x] = kk;
        ProgramCounterState::Next
    }

    fn add_kk(&mut self, x: usize, kk: u8) -> ProgramCounterState {
        self.v[x] = self.v[x].wrapping_add(kk);
        ProgramCounterState::Next
    }

    fn load_vy(&mut self, x: usize, y: usize) -> ProgramCounterState {
        self.v[x] = self.v[y];
        ProgramCounterState::Next
    }

    fn or(&mut self, x: usize, y: usize) -> ProgramCounterState {
        self.v[x] |= self.v[y];
        self.v[0xf] = 0;
        ProgramCounterState::Next
    }

    fn and(&mut self, x: usize, y: usize) -> ProgramCounterState {
        self.v[x] &= self.v[y];
        self.v[0xf] = 0;
        ProgramCounterState::Next
    }

    fn xor(&mut self, x: usize, y: usize) -> ProgramCounterState {
        self.v[x] ^= self.v[y];
        self.v[0xf] = 0;
        ProgramCounterState::Next
    }

    fn add_vy(&mut self, x: usize, y: usize) -> ProgramCounterState {
        let result = self.v[x] as u16 + self.v[y] as u16;
        self.v[x] = result as u8;
        self.v[0xf] = (result > 0xff) as u8;
        ProgramCounterState::Next
    }

    fn sub(&mut self, x: usize, y: usize) -> ProgramCounterState {
        let result = self.v[x] as i16 - self.v[y] as i16;
        self.v[x] = result as u8;
        self.v[0xf] = (result >= 0) as u8;
        ProgramCounterState::Next
    }

    fn shr(&mut self, x: usize, y: usize) -> ProgramCounterState {
        let original = self.v[y];
        self.v[x] = self.v[y] >> 1;
        self.v[0xf] = original & 1;
        ProgramCounterState::Next
    }

    fn subn(&mut self, x: usize, y: usize) -> ProgramCounterState {
        let result = self.v[y] as i16 - self.v[x] as i16;
        self.v[x] = result as u8;
        self.v[0xf] = (result >= 0) as u8;
        ProgramCounterState::Next
    }

    fn shl(&mut self, x: usize, y: usize) -> ProgramCounterState {
        let original = self.v[y];
        self.v[x] = self.v[y] << 1;
        self.v[0xf] = (original & 0b10000000) >> 7;
        ProgramCounterState::Next
    }

    fn skip_vy_ne(&mut self, x: usize, y: usize) -> ProgramCounterState {
        if self.v[x] != self.v[y] {
            return ProgramCounterState::Skip;
        }
        ProgramCounterState::Next
    }

    fn load_addr(&mut self, nnn: usize) -> ProgramCounterState {
        self.i = nnn;
        ProgramCounterState::Next
    }

    fn rand(&mut self, x: usize, kk: u8) -> ProgramCounterState {
        let num: u8 = rand::rng().random();
        self.v[x] = num & kk;
        ProgramCounterState::Next
    }

    fn draw(&mut self, x: usize, y: usize, n: usize) -> ProgramCounterState {
        let x = (self.v[x] as usize) % CHIP8_WIDTH;
        let y = (self.v[y] as usize) % CHIP8_HEIGHT;
        self.v[0xf] = 0;

        'outer: for i in 0..n {
            let pixel = self.mem[self.i + i];
            for j in 0..8 {
                let val = (pixel >> (7 - j)) & 0b1;
                let x = x + j;
                let y = y + i;
                if y >= CHIP8_HEIGHT {
                    break 'outer;
                }
                if x >= CHIP8_WIDTH {
                    continue;
                }
                self.v[0xf] = val & self.screen[y][x];
                self.screen[y][x] ^= val;
            }
        }

        self.draw_flag = true;

        ProgramCounterState::Next
    }

    fn skip_key_eq(&mut self, x: usize) -> ProgramCounterState {
        if self.keypad[self.v[x] as usize] {
            return ProgramCounterState::Skip;
        }
        ProgramCounterState::Next
    }

    fn skip_key_ne(&mut self, x: usize) -> ProgramCounterState {
        if !self.keypad[self.v[x] as usize] {
            return ProgramCounterState::Skip;
        }
        ProgramCounterState::Next
    }

    fn load_delay(&mut self, x: usize) -> ProgramCounterState {
        self.v[x] = self.delay;
        ProgramCounterState::Next
    }

    fn load_key(&mut self, x: usize) -> ProgramCounterState {
        self.waiting_key = true;
        if let None = self.last_key {
            self.pc -= 2; // rerun this
            return ProgramCounterState::Next;
        }

        let code = self.last_key.unwrap();
        self.waiting_key = false;
        self.last_key = None;
        self.v[x] = code;

        ProgramCounterState::Next
    }

    fn load_vx_delay(&mut self, x: usize) -> ProgramCounterState {
        self.delay = self.v[x];
        ProgramCounterState::Next
    }

    fn load_vx_sound(&mut self, x: usize) -> ProgramCounterState {
        self.sound = self.v[x];
        ProgramCounterState::Next
    }

    fn add_i(&mut self, x: usize) -> ProgramCounterState {
        self.i += self.v[x] as usize;
        ProgramCounterState::Next
    }

    fn load_sprite(&mut self, x: usize) -> ProgramCounterState {
        self.i = (self.v[x] as usize) * 5;
        ProgramCounterState::Next
    }

    fn load_bcd(&mut self, x: usize) -> ProgramCounterState {
        let num = self.v[x];
        self.mem[self.i] = num / 100;
        self.mem[self.i + 1] = (num / 10) % 10;
        self.mem[self.i + 2] = num % 10;
        ProgramCounterState::Next
    }

    fn store_v0_vx(&mut self, x: usize) -> ProgramCounterState {
        for i in 0..x + 1 {
            self.mem[self.i] = self.v[i];
            self.i += 1;
        }
        ProgramCounterState::Next
    }

    fn load_v0_vx(&mut self, x: usize) -> ProgramCounterState {
        for i in 0..x + 1 {
            self.v[i] = self.mem[self.i];
            self.i += 1;
        }
        ProgramCounterState::Next
    }
}
