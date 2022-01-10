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
    let mut split = false;

    // parse argument
    for arg in env::args() {
        if arg == "-s" {
            split = true;
        }
    }

    let stdin = io::stdin();
    for ln in stdin.lock().lines() {
        let line = ln.unwrap();
        
        if line.len() < 4 {
            continue;
        }

        let status_code = &line[0..2];
        let sep = &line[2..3];
        let path_line = &line[3..];

        if sep != " " {
            continue;
        }

        let _status_code = &str::replace(&status_code, " ", "_");
        let ch1 = _status_code.chars().nth(0).unwrap();
        let ch2 = _status_code.chars().nth(1).unwrap();

        let mut entry: [&str; 3] = ["", "", ""];
        let iter = str::split_whitespace(path_line);
        let mut idx = 0;
        for e in iter {
            entry[idx] = e;
            idx = idx + 1;
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
