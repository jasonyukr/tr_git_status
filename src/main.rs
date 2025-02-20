use std::io::{self, BufRead, BufWriter, Write};
use std::env;
use regex::Regex;

fn get_deco_status_code(ch1: char, ch2: char) -> String {
    if ch1 == '?' && ch2 == '?' {
       return String::from(format!("\x1b[31m{}{}", ch1, ch2));
    } else {
       return String::from(format!("\x1b[32m{}\x1b[31m{}", ch1, ch2));
    }
}

fn process_line<W: Write>(re: &Regex, out: &mut BufWriter<W>, split: bool, line: &str) -> bool {
    if line.len() < 4 {
        return false;
    }

    /*
     * Parse the line.
     *   XY pathname
     *   XY pathname1 -> pathname2
     *   XY "pathname"
     *   XY "pathname1" -> "pathname2"
     */
    let mut status_code = &line[0..2];
    let sep = &line[2..3];
    let path_line = &line[3..];
    if sep != " " {
        return false;
    }

    // Replace the space with "_" in the status code (XY)
    let binding = &status_code.replace(" ", "_");
    status_code = binding;
    let ch1 = status_code.chars().nth(0).unwrap();
    let ch2 = status_code.chars().nth(1).unwrap();
    let deco_status_code = get_deco_status_code(ch1, ch2);

    // Parse the pathname part
    if let Some(caps) = re.captures(path_line) {
        let path1 = caps.get(1).map_or("", |m| m.as_str());
        let path2 = caps.get(2).map_or("", |m| m.as_str());

        if split {
            writeln!(out, "{} \x1b[0m{} :: \x1b[33m{}\x1b[0m", deco_status_code, path_line, path1).unwrap();
            writeln!(out, "{} \x1b[0m{} :: \x1b[33m{}\x1b[0m", deco_status_code, path_line, path2).unwrap();
        } else {
            writeln!(out, "{} \x1b[0m{} -> \x1b[33m{}\x1b[0m", deco_status_code, path1, path2).unwrap();
        }
    } else {
        writeln!(out, "{} \x1b[33m{}\x1b[0m", deco_status_code, path_line).unwrap();
    }

    true
}

fn main() {
    // parse argument
    let mut split = false;
    for arg in env::args() {
        if arg == "-s" {
            split = true;
        }
    }

    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout);

    /*
     * regular expression for
     *   A -> B
     *   A -> "B"
     *   "A" -> B
     *   "A" -> "B"
     */
    let re = Regex::new(r#""?(.*?)"?\s->\s"?([^"]*)"?"#).unwrap();

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        if let Ok(ln) = line {
            process_line(&re, &mut out, split, &ln);
        }
    }
    out.flush().unwrap();
}
