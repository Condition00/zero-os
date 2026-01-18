use alloc::string::{String, ToString};
use spin::Mutex;

static INPUT_BUFFER: Mutex<String> = Mutex::new(String::new());

pub fn push_char(c: char) {
    let mut buf = INPUT_BUFFER.lock();

    match c {
        '\n' => buf.push('\n'),
        '\x08' => {
            // backspace
            buf.pop();
        }
        _ => buf.push(c),
    }
}

pub async fn read_line() -> String {
    loop {
        {
            let mut buf = INPUT_BUFFER.lock();
            if let Some(pos) = buf.find('\n') {
                let line = buf[..pos].to_string();
                *buf = buf[pos + 1..].to_string();
                return line;
            }
        }
        crate::task::yield_now().await;
    }
}
