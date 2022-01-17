use std::io::{self, BufRead};
use std::env;

fn get_deco_status_code(ch1: char, ch2: char) -> String {
    if ch1 == '?' && ch2 == '?' {
       return String::from(format!("\x1b[31m{}{}", ch1, ch2));
    } else {
       return String::from(format!("\x1b[32m{}\x1b[31m{}", ch1, ch2));
    }
}

fn main() {
    // parse argument
    let mut split = false;
    for arg in env::args() {
        if arg == "-s" {
            split = true;
        }
    }

    let stdin = io::stdin();
    for ln in stdin.lock().lines() {
        let line;
        match ln {
            Ok(data) => line = data,
            Err(_) => continue
        }
        
        if line.len() < 4 {
            continue;
        }

        let status_code = &line[0..2];
        let sep = &line[2..3];
        let path_line = &line[3..];

        if sep != " " {
            continue;
        }

        let ch1;
        if let Some(ch) = status_code.chars().nth(0) {
            if ch == ' ' {
                ch1 = '_';
            } else {
                ch1 = ch;
            }
        } else {
            continue;
        }

        let ch2;
        if let Some(ch) = status_code.chars().nth(1) {
            if ch == ' ' {
                ch2 = '_';
            } else {
                ch2 = ch;
            }
        } else {
            continue;
        }

        let mut entry = ["", "", ""];
        for (idx, e) in str::split_whitespace(path_line).enumerate() {
            entry[idx] = e;
            if idx >= 3 {
                break;
            }
        }

        let deco_status_code = get_deco_status_code(ch1, ch2);

        if entry[1] == "->" {
            if split {
                println!("{} \x1b[0m{} :: \x1b[33m{}\x1b[0m", deco_status_code, path_line, entry[0]);
                println!("{} \x1b[0m{} :: \x1b[33m{}\x1b[0m", deco_status_code, path_line, entry[2]);
            } else {
                println!("{} \x1b[0m{} -> \x1b[33m{}\x1b[0m", deco_status_code, entry[0], entry[2]);
            }
        } else {
            println!("{} \x1b[33m{}\x1b[0m", deco_status_code, path_line);
        }
    }
}
