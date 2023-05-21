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
        println!("Command: {}", self.command);
        println!("Args: {:?}", self.args);
        self.to_c_command();
    }

    fn to_c_command(&self) {
        let prog = CString::new(self.command.as_str()).unwrap();

        let mut args = match &self.args {
            Some(args) => args
                .iter()
                .map(|arg| CString::new(arg.as_str()).unwrap())
                .collect(),
            None => {
                vec![]
            }
        };

        args.insert(0, clean_prog(self.command.clone()));

        let argv: Vec<*const c_char> = args.iter().map(|arg| arg.as_ptr()).collect();
        let envp: *const *const c_char = std::ptr::null();

        let mut fds: [c_int; 2] = [0, 0];

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
                libc::dup2(fds[1], libc::STDOUT_FILENO);
                libc::close(fds[1]);

                let rs = libc::execve(prog.as_ptr(), argv.as_ptr(), envp);
                println!("rs: {}", rs);
            }else{
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

fn clean_prog(command: String) -> CString {
    let mut prog = command;
    if prog.contains("/") {
        prog = prog.split("/").last().unwrap().to_string();
    }
    CString::new(prog.as_str()).unwrap()
}
