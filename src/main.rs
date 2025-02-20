use std::io::{self, BufRead, BufWriter, Write};
use std::env;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /*
     * regular expression for matching
     *   A -> B
     *   A -> "B"
     *   "A" -> B
     *   "A" -> "B"
     */
    static ref re: Regex = Regex::new(r#""?(.*?)"?\s->\s"?([^"]*)"?"#).unwrap();
}

fn get_deco_status_code(ch1: char, ch2: char) -> String {
    if ch1 == '?' && ch2 == '?' {
       return String::from(format!("\x1b[31m{}{}", ch1, ch2));
    } else {
       return String::from(format!("\x1b[32m{}\x1b[31m{}", ch1, ch2));
    }
}

fn parse_rename(line: &str) -> Option<(&str, &str)> {
    if let Some(caps) = re.captures(line) {
        let path1 = caps.get(1).map_or("", |m| m.as_str());
        let path2 = caps.get(2).map_or("", |m| m.as_str());
        Some((path1, path2))
    } else {
        None
    }
}

#[test]
fn test_parse_rename() {
    assert_eq!(parse_rename(r#"AAA.txt"#), None);
    assert_eq!(parse_rename(r#""AAA space.txt""#), None);
    assert_eq!(parse_rename(r#"AAA.txt->CCC.txt"#), None);
    assert_eq!(parse_rename(r#""AAA space.txt" -> "CCC space.txt""#), Some(("AAA space.txt", "CCC space.txt")));
    assert_eq!(parse_rename(r#""AAA space.txt" -> CCC.txt"#), Some(("AAA space.txt", "CCC.txt")));
    assert_eq!(parse_rename(r#"AAA.txt -> "CCC space.txt""#), Some(("AAA.txt", "CCC space.txt")));
    assert_eq!(parse_rename(r#"AAA.txt -> CCC.txt"#), Some(("AAA.txt", "CCC.txt")));
}

fn add_quotes_if_with_whitespace(s: &str) -> String {
    if s.contains(char::is_whitespace) {
        format!("\"{}\"", s)
    } else {
        s.to_string()
    }
}

fn process_line<W: Write>(out: &mut BufWriter<W>, split: bool, line: &str) -> bool {
    if line.len() < 4 {
        return false;
    }

    /*
     * Parse the line input
     *   XY pathname
     *   XY pathname1 -> pathname2
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

    // Parse the pathname part (generic or rename case)
    if let Some((path1, path2)) = parse_rename(path_line) {
        // rename case
        let path1q = add_quotes_if_with_whitespace(path1);
        let path2q = add_quotes_if_with_whitespace(path2);
        if split {
            writeln!(out, "{} \x1b[0m{} :: \x1b[33m{}\x1b[0m", deco_status_code, path_line, path1q).unwrap();
            writeln!(out, "{} \x1b[0m{} :: \x1b[33m{}\x1b[0m", deco_status_code, path_line, path2q).unwrap();
        } else {
            writeln!(out, "{} \x1b[0m{} -> \x1b[33m{}\x1b[0m", deco_status_code, path1q, path2q).unwrap();
        }
    } else {
        // generic case
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

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        if let Ok(ln) = line {
            process_line(&mut out, split, &ln);
        }
    }
    out.flush().unwrap();
}
