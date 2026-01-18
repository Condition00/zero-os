use crate::arch;
use crate::{input, terminal};
use alloc::string::String;
use alloc::vec::Vec;

pub async fn shell() {
    loop {
        terminal::write("zero-os> ");
        terminal::mark_input_start();

        let line = input::read_line().await;
        run_command(line);
    }
}

fn run_command(line: String) {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    match parts[0] {
        "help" => cmd_help(),
        "clear" => terminal::clear(),
        "echo" => cmd_echo(&parts[1..]),
        "reboot" => cmd_reboot(),
        _ => {
            terminal::write("command not found\n");
        }
    }
}

fn cmd_help() {
    terminal::write("Available commands:\n");
    terminal::write("  help   - show this message\n");
    terminal::write("  clear  - clear screen\n");
    terminal::write("  echo   - print text\n");
    terminal::write("  reboot - reboot machine\n");
}

fn cmd_echo(args: &[&str]) {
    for arg in args {
        terminal::write(arg);
        terminal::write(" ");
    }
    terminal::write("\n");
}

fn cmd_reboot() {
    terminal::write("Rebooting...\n");
    arch::reboot();
}
