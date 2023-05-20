use std::{ffi::CString, os::raw::c_char};

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
        let mut args = vec![
            CString::new(self.command.as_str()).unwrap(),
        ];

        if let Some(self_args) = &self.args {
            for arg in self_args {
                args.push(CString::new(arg.as_str()).unwrap());
            }
        }

        let argv: Vec<*const c_char> = args.iter().map(|arg| arg.as_ptr()).collect();
        let envp: *const *const c_char = std::ptr::null();

        unsafe {
            let rs = libc::execve(argv[0], argv.as_ptr(), envp);
            println!("rs: {}", rs);
        }
    }
}
