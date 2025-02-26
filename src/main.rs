use std::io::{self, BufRead, BufWriter, Write};
use std::env;
use lazy_static::lazy_static;
use regex::Regex;
use std::path::Path;
use lscolors::{LsColors, Style};

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

#[cfg(all(
    not(feature = "nu-ansi-term"),
))]
compile_error!(
    "feature must be enabled: nu-ansi-term"
);

fn print_lscolor_path(handle: &mut dyn Write, ls_colors: &LsColors, path: &str) -> io::Result<()> {
    for (component, style) in ls_colors.style_for_path_components(Path::new(path)) {
        #[cfg(any(feature = "nu-ansi-term", feature = "gnu_legacy"))]
        {
            let ansi_style = style.map(Style::to_nu_ansi_term_style).unwrap_or_default();
            write!(handle, "{}", ansi_style.paint(component.to_string_lossy()))?;
        }
    }
    Ok(())
}

fn get_deco_status_code(ch1: char, ch2: char) -> String {
    if ch1 == '?' && ch2 == '?' {
       return String::from(format!("\x1b[31m{}{}\x1b[0m", ch1, ch2));
    } else {
       return String::from(format!("\x1b[32m{}\x1b[31m{}\x1b[0m", ch1, ch2));
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

fn process_line<W: Write>(out: &mut BufWriter<W>, ls_colors: &LsColors, split: bool, line: &str) -> bool {
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
            write!(out, "{} {} :: ", deco_status_code, path_line).unwrap();
            print_lscolor_path(out, &ls_colors, &path1q).unwrap();
            writeln!(out).unwrap();

            write!(out, "{} {} :: ", deco_status_code, path_line).unwrap();
            print_lscolor_path(out, &ls_colors, &path2q).unwrap();
            writeln!(out).unwrap();
        } else {
            write!(out, "{} ", deco_status_code).unwrap();
            print_lscolor_path(out, &ls_colors, &path1q).unwrap();
            write!(out, " -> ").unwrap();
            print_lscolor_path(out, &ls_colors, &path2q).unwrap();
            writeln!(out).unwrap();
        }
    } else {
        // generic case
        write!(out, "{} ", deco_status_code).unwrap();
        print_lscolor_path(out, &ls_colors, path_line).unwrap();
        writeln!(out).unwrap();
    }

    true
}

fn main() {
    let ls_colors = LsColors::from_env().unwrap_or_default();

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
            process_line(&mut out, &ls_colors, split, &ln);
        }
    }
    out.flush().unwrap();
}
