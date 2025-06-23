mod chip8;

fn main() {
    let mut chip8 = chip8::Chip8::new();
    let op = chip8.get_op();
    chip8.exec_op(op);
}
