#![deny(warnings)]

extern crate syscall;

use std::process::Command;
use std::{env, process, str};
use std::io::{self, Write};

fn set_tty(tty: &str) -> syscall::Result<()> {
    let stdin = syscall::open(tty, syscall::flag::O_RDONLY)?;
    let stdout = syscall::open(tty, syscall::flag::O_WRONLY)?;
    let stderr = syscall::open(tty, syscall::flag::O_WRONLY)?;

    syscall::dup2(stdin, 0, &[])?;
    syscall::dup2(stdout, 1, &[])?;
    syscall::dup2(stderr, 2, &[])?;

    let _ = syscall::close(stdin);
    let _ = syscall::close(stdout);
    let _ = syscall::close(stderr);

    Ok(())
}

fn daemon(clear: bool) {
    loop {
        if clear {
            syscall::write(1, b"\x1Bc").unwrap();
        }
        syscall::fsync(1).unwrap();
        match Command::new("login").spawn() {
            Ok(mut child) => match child.wait() {
                Ok(_status) => (), //println!("getty: waited for login: {:?}", status.code()),
                Err(err) => panic!("getty: failed to wait for login: {}", err)
            },
            Err(err) => panic!("getty: failed to execute login: {}", err)
        }
    }
}

pub fn main() {
    let mut tty_option = None;
    let mut clear = true;
    for arg in env::args().skip(1) {
        match arg.as_str() {
            "-J" | "--noclear" => {
                clear = false;
            },
            _ => {
                tty_option = Some(arg);
            }
        }
    }

    if let Some(tty) = tty_option {
        if let Err(err) = set_tty(&tty) {
            writeln!(io::stderr(), "getty: failed to open TTY {}: {}", tty, err).unwrap();
            process::exit(1);
        }

        env::set_var("TTY", &tty);
        {
            let mut path = [0; 4096];
            if let Ok(count) = syscall::fpath(0, &mut path) {
                let path_str = str::from_utf8(&path[..count]).unwrap_or("");
                let reference = path_str.split(':').nth(1).unwrap_or("");
                let mut parts = reference.split('/').skip(1);
                env::set_var("COLUMNS", parts.next().unwrap_or("80"));
                env::set_var("LINES", parts.next().unwrap_or("30"));
            } else {
                env::set_var("COLUMNS", "80");
                env::set_var("LINES", "30");
            }
        }

        if unsafe { syscall::clone(0).unwrap() } == 0 {
            daemon(clear);
        }
    } else {
        panic!("getty: no tty provided");
    }
}
