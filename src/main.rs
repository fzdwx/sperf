use std::{ffi::CString, os::raw::c_char, vec};

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
                .map(|arg| {
                    return CString::new(arg.as_str()).unwrap();
                })
                .collect(),
            None => {
                vec![]
            }
        };

        args.insert(0, clean_prog(self.command.clone()));

        let argv: Vec<*const c_char> = args.iter().map(|arg| arg.as_ptr()).collect();
        let envp: *const *const c_char = std::ptr::null();

        unsafe {
            let rs = libc::execve(prog.as_ptr(), argv.as_ptr(), envp);
            println!("rs: {}", rs);
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
