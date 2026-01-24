use crate::vg_buffer::WRITER;
use core::fmt::Write;
use spin::Mutex;

static INPUT_START: Mutex<usize> = Mutex::new(0);

pub fn mark_input_start() {
    let col = crate::vg_buffer::WRITER.lock().column();
    *INPUT_START.lock() = col;
}

pub fn can_backspace() -> bool {
    let col = crate::vg_buffer::WRITER.lock().column();
    let start = *INPUT_START.lock();
    col > start
}

pub fn write(s: &str) {
    WRITER.lock().write_str(s).unwrap();
}

pub fn write_char(c: char) {
    let mut buf = [0; 4];
    write(c.encode_utf8(&mut buf));
}

pub fn clear() {
    let mut writer = WRITER.lock();
    for _ in 0..25 {
        writer.write_str("\n").unwrap();
    }
}

pub fn backspace() {
    if !can_backspace() {
        return;
    }

    crate::vg_buffer::WRITER.lock().backspace();
}
