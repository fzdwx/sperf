#[warn(unused_assignments)]
use std::{
    ffi::CString,
    os::raw::{c_char, c_int},
    vec,
};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::{Deref};
use chrono::Local;

use clap::Parser;
use libc::{execve};

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

        args.insert(0, CString::new("strace").unwrap());
        args.insert(1, CString::new("-T").unwrap());
        args.insert(2, find_command_path(self.command.clone()));

        let mut argv: Vec<*const c_char> = args.iter().map(|arg| arg.as_ptr()).collect();
        argv.push(std::ptr::null());

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

                if execve(prog.as_ptr(), argv.as_ptr(), std::ptr::null()) == -1 {
                    panic!("Failed to execve");
                }
            } else {
                libc::close(fds[1]);
                let mut buf = [0u8; 1024];
                let mut n = 0;

                let mut string = String::new();
                let mut last = Local::now().timestamp();

                let mut syscall_map: HashMap<String, Syscall> = HashMap::new();
                loop {
                    n = libc::read(fds[0], buf.as_mut_ptr() as *mut libc::c_void, buf.len());
                    if n == 0 {
                        parse(string, &mut syscall_map);
                        result(&syscall_map);
                        break;
                    }

                    let s = String::from_utf8_lossy(&buf[..n as usize]);
                    string.push_str(s.deref());

                    if &last - &Local::now().timestamp() > 1 {
                        parse(string,&mut syscall_map);
                        string = String::new();
                        last = Local::now().timestamp();
                    }
                }

                libc::close(fds[0]);
            }
        }
    }
}

fn result(s: &HashMap<String, Syscall>) {
    let mut vec: Vec<&Syscall> = s.values().collect();
    vec.sort_by(|a, b| b.cost.partial_cmp(&a.cost).unwrap());
    vec.iter().for_each(|syscall| {
        println!("{}", syscall);
    });
}

fn parse(s: String, map: &mut HashMap<String, Syscall>) {
    let mut line = String::new();
    s.chars().for_each(|c| {
        if c == '\n' {
            parse_line(&line).map(|syscall| {
                if let Some(pre_syscall) = map.get_mut(&syscall.name) {
                    pre_syscall.cost += syscall.cost;
                    pre_syscall.count += 1;
                } else {
                    map.insert(syscall.name.to_string(), syscall);
                }
            });
            line = String::new();
        } else {
            line.push(c);
        }
    });
}

fn parse_line(text: &str) -> Option<Syscall> {
    let mut name = String::new();
    let mut time = String::new();
    for (_, ch) in text.chars().enumerate() {
        if ch == '(' {
            break;
        }
        name.push(ch);
    }
    if name == "read" {
        println!("{}", text)
    }
    let mut start = false;
    for (_, ch) in text.chars().enumerate() {
        if ch == '<' {
            start = true;
            continue;
        }
        if ch == '>' && start == true {
            break;
        }
        if start {
            time.push(ch);
        }
    }
    if name.is_empty() || time.is_empty() {
        return None;
    }
    Some(Syscall::new(name, time))
}

fn find_command_path(command: String) -> CString {
    let prog = command;
    if prog.starts_with("/") {
        return wrap(prog);
    }
    CString::new(prog.as_str()).unwrap()
}

fn wrap(prog: String) -> CString {
    CString::new(prog.as_str()).unwrap()
}

struct Syscall {
    name: String,
    cost: f64,
    count: u64,
}

impl Display for Syscall {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}



impl Syscall {
    fn new(name: String, cost: String) -> Self {
        Self { name, cost: cost.parse().unwrap(), count: 1 }
    }

    pub fn to_string(&self) -> String {
        format!("{}: called {}, cost {}s", self.name, self.count, self.cost)
    }
}
