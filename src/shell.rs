use crate::arch;
use crate::fs;
use crate::{input, terminal};
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

pub async fn shell() {
    loop {
        terminal::write("user@zero-os:-# ");
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
        "ls" => cmd_ls(&parts[1..]),
        "cat" => cmd_cat(&parts[1..]),
        "mkdir" => cmd_mkdir(&parts[1..]),
        "touch" => cmd_touch(&parts[1..]),
        "rm" => cmd_rm(&parts[1..]),
        "write" => cmd_write(&parts[1..]),
        "stat" => cmd_stat(&parts[1..]),
        _ => {
            terminal::write("command not found\n");
        }
    }
}

fn cmd_help() {
    terminal::write("Available commands:\n");
    terminal::write("  help         - show this message\n");
    terminal::write("  clear        - clear screen\n");
    terminal::write("  echo <text>  - print text\n");
    terminal::write("  reboot       - reboot machine\n");
    terminal::write("  ls [path]    - list directory contents\n");
    terminal::write("  cat <file>   - display file contents\n");
    terminal::write("  mkdir <dir>  - create directory\n");
    terminal::write("  touch <file> - create empty file\n");
    terminal::write("  rm <path>    - remove file or empty directory\n");
    terminal::write("  write <file> <text> - write text to file\n");
    terminal::write("  stat <path>  - show file/directory information\n");
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

fn cmd_ls(args: &[&str]) {
    let path = if args.is_empty() { "/" } else { args[0] };

    if let Some(fs) = fs::root() {
        match fs.list_dir(path) {
            Ok(entries) => {
                if entries.is_empty() {
                    terminal::write("(empty)\n");
                } else {
                    for entry in entries {
                        let type_char = match entry.file_type {
                            fs::FileType::Directory => 'd',
                            fs::FileType::File => 'f',
                        };
                        let output = format!("{} {:8} {}\n", type_char, entry.size, entry.name);
                        terminal::write(&output);
                    }
                }
            }
            Err(e) => {
                let msg = format!("ls: {}\n", e);
                terminal::write(&msg);
            }
        }
    } else {
        terminal::write("filesystem not initialized\n");
    }
}

fn cmd_cat(args: &[&str]) {
    if args.is_empty() {
        terminal::write("cat: missing file operand\n");
        return;
    }

    if let Some(fs) = fs::root() {
        match fs.read_file(args[0]) {
            Ok(data) => match core::str::from_utf8(&data) {
                Ok(s) => {
                    terminal::write(s);
                    terminal::write("\n");
                }
                Err(_) => {
                    terminal::write("cat: file contains non-UTF8 data\n");
                }
            },
            Err(e) => {
                let msg = format!("cat: {}\n", e);
                terminal::write(&msg);
            }
        }
    } else {
        terminal::write("filesystem not initialized\n");
    }
}

fn cmd_mkdir(args: &[&str]) {
    if args.is_empty() {
        terminal::write("mkdir: missing operand\n");
        return;
    }

    if let Some(fs) = fs::root() {
        match fs.create_dir(args[0]) {
            Ok(_) => {}
            Err(e) => {
                let msg = format!("mkdir: {}\n", e);
                terminal::write(&msg);
            }
        }
    } else {
        terminal::write("filesystem not initialized\n");
    }
}

fn cmd_touch(args: &[&str]) {
    if args.is_empty() {
        terminal::write("touch: missing operand\n");
        return;
    }

    if let Some(fs) = fs::root() {
        match fs.create_file(args[0]) {
            Ok(_) => {}
            Err(e) => {
                let msg = format!("touch: {}\n", e);
                terminal::write(&msg);
            }
        }
    } else {
        terminal::write("filesystem not initialized\n");
    }
}

fn cmd_rm(args: &[&str]) {
    if args.is_empty() {
        terminal::write("rm: missing operand\n");
        return;
    }

    if let Some(fs) = fs::root() {
        match fs.remove(args[0]) {
            Ok(_) => {}
            Err(e) => {
                let msg = format!("rm: {}\n", e);
                terminal::write(&msg);
            }
        }
    } else {
        terminal::write("filesystem not initialized\n");
    }
}

fn cmd_write(args: &[&str]) {
    if args.len() < 2 {
        terminal::write("write: missing operands\n");
        terminal::write("usage: write <file> <text>\n");
        return;
    }

    let filename = args[0];
    let text = args[1..].join(" ");

    if let Some(fs) = fs::root() {
        // Create file if it doesn't exist
        if !fs.exists(filename) {
            if let Err(e) = fs.create_file(filename) {
                let msg = format!("write: {}\n", e);
                terminal::write(&msg);
                return;
            }
        }

        match fs.write_file(filename, text.as_bytes()) {
            Ok(_) => {}
            Err(e) => {
                let msg = format!("write: {}\n", e);
                terminal::write(&msg);
            }
        }
    } else {
        terminal::write("filesystem not initialized\n");
    }
}

fn cmd_stat(args: &[&str]) {
    if args.is_empty() {
        terminal::write("stat: missing operand\n");
        return;
    }

    if let Some(fs) = fs::root() {
        match fs.stat(args[0]) {
            Ok(info) => {
                let type_str = match info.file_type {
                    fs::FileType::Directory => "directory",
                    fs::FileType::File => "file",
                };
                terminal::write("  File: ");
                terminal::write(&info.name);
                terminal::write("\n  Type: ");
                terminal::write(type_str);
                terminal::write("\n  Size: ");
                let size_str = format!("{} bytes\n", info.size);
                terminal::write(&size_str);
            }
            Err(e) => {
                let msg = format!("stat: {}\n", e);
                terminal::write(&msg);
            }
        }
    } else {
        terminal::write("filesystem not initialized\n");
    }
}
