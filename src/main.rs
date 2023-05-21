use std::{
    ffi::CString,
    os::raw::{c_char, c_int},
    vec,
};

use clap::Parser;
use libc::execve;

fn main() {
    Cli::parse().run();
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Command name or command alias or command path
    command: String,
    /// command arguments
    args: Option<Vec<String>>,
}

impl Cli {
    pub fn run(&self) {
        let prog = CString::new("/usr/bin/strace").unwrap();

        let mut args = match &self.args {
            Some(args) => args
                .iter()
                .map(|arg| CString::new(arg.as_str()).unwrap())
                .collect(),
            None => {
                vec![]
            }
        };

        args.insert(0, prog.clone());
        args.insert(1, find_command_path(self.command.clone()));

        let argv: Vec<*const c_char> = args.iter().map(|arg| arg.as_ptr()).collect();
        let envp: Vec<*const c_char> = vec![CString::new("").unwrap()]
            .iter()
            .map(|arg| arg.as_ptr())
            .collect::<Vec<*const c_char>>();

        let mut fds: [c_int; 2] = [libc::STDERR_FILENO, libc::STDOUT_FILENO]; // 0 is read, 1 is write

        unsafe {
            if libc::pipe(fds.as_mut_ptr()) == -1 {
                panic!("Failed to create pipe");
            }

            let pid = libc::fork();
            if pid == -1 {
                panic!("Failed to fork");
            }

            if pid == 0 {
                // child
                libc::close(fds[0]);
                libc::dup2(fds[1], libc::STDERR_FILENO);
                libc::close(fds[1]);

                if libc::execve(prog.as_ptr(), argv.as_ptr(), envp.as_ptr()) == -1 {
                    panic!("Failed to execve");
                }
            } else {
                libc::close(fds[1]);
                let mut buf = [0u8; 1024];
                let mut n = 0;
                loop {
                    n = libc::read(fds[0], buf.as_mut_ptr() as *mut libc::c_void, buf.len());
                    if n == 0 {
                        break;
                    }
                    String::from_utf8_lossy(&buf).lines().for_each(|line| {
                        println!("line: {}", line);
                    });
                }
                libc::close(fds[0]);
            }
        }
    }
}

fn find_command_path(command: String) -> CString {
    let mut prog = command;
    if prog.starts_with("/") {
        return wrap(prog);
    }
    CString::new(prog.as_str()).unwrap()
}

fn wrap(prog: String) -> CString {
    CString::new(prog.as_str()).unwrap()
}
