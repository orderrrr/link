#![allow(unused_imports, unused_variables)]

use std::borrow::Cow;

use rustyline::completion::Completer;
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Editor, Helper};

use cfg_if::cfg_if;

use l::byte::I;
use l::vm::V;

// ---------------------------------------------------------------------------
// ANSI color codes
// ---------------------------------------------------------------------------
const RESET: &str = "\x1b[0m";
const CYAN: &str = "\x1b[36m"; // operators
const MAGENTA: &str = "\x1b[35m"; // combinators
const YELLOW: &str = "\x1b[33m"; // numbers
const GREEN: &str = "\x1b[32m"; // strings
const BLUE: &str = "\x1b[34m"; // brackets
const BOLD: &str = "\x1b[1m"; // assignment
const GREY: &str = "\x1b[90m"; // comments, pipes

// ---------------------------------------------------------------------------
// Alias replacement: ASCII names → unicode symbols
// ---------------------------------------------------------------------------

/// (ascii name, unicode replacement)
/// Longer names must come first to avoid partial matches
/// (e.g. "scanl" before "scan")
const ALIASES: &[(&str, &str)] = &[
    ("max", "¯"),
    ("min", "_"),
    ("mod", "!"),
    ("mul", "×"),
    ("div", "÷"),
    ("eq", "="),
    ("amp", "&"),
    ("rho", "ρ"),
    ("each", "ǁ"),
    ("fold", "/"),
    ("scanl", "\\"),
];

/// Replace all ASCII aliases with their unicode counterparts.
/// Only replaces when the alias appears as a standalone word
/// (not inside a string literal or a longer identifier).
fn replace_aliases(input: &str) -> String {
    let mut result = input.to_string();
    for &(name, symbol) in ALIASES {
        let mut out = String::with_capacity(result.len());
        let chars: Vec<char> = result.chars().collect();
        let len = chars.len();
        let name_chars: Vec<char> = name.chars().collect();
        let name_len = name_chars.len();
        let mut i = 0;
        let mut in_string = false;

        while i < len {
            // Track string literals — don't replace inside them
            if chars[i] == '"' {
                in_string = !in_string;
                out.push(chars[i]);
                i += 1;
                continue;
            }

            if in_string {
                out.push(chars[i]);
                i += 1;
                continue;
            }

            // Check if the alias matches at this position
            if i + name_len <= len {
                let slice_matches = (0..name_len).all(|j| chars[i + j] == name_chars[j]);

                if slice_matches {
                    // Check that it's not part of a longer identifier:
                    // char before must not be a letter or underscore
                    let before_ok =
                        i == 0 || !(chars[i - 1].is_alphabetic() || chars[i - 1] == '_');
                    // char after must not be a letter or underscore
                    // (digits are fine — rho5 should become ρ5)
                    let after_ok = i + name_len >= len
                        || !(chars[i + name_len].is_alphabetic() || chars[i + name_len] == '_');

                    if before_ok && after_ok {
                        out.push_str(symbol);
                        i += name_len;
                        continue;
                    }
                }
            }

            out.push(chars[i]);
            i += 1;
        }

        result = out;
    }
    result
}

// ---------------------------------------------------------------------------
// Syntax highlighter
// ---------------------------------------------------------------------------

struct LinkHighlighter;

impl LinkHighlighter {
    fn highlight_line(&self, line: &str) -> String {
        let mut out = String::with_capacity(line.len() * 2);
        let chars: Vec<char> = line.chars().collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            let ch = chars[i];

            // Check for alias keywords — color them but keep original text
            // (rustyline requires highlight output to have same display width as input)
            if ch.is_ascii_alphabetic() {
                if let Some((name, symbol)) = ALIASES.iter().find(|(name, _)| {
                    let name_chars: Vec<char> = name.chars().collect();
                    let name_len = name_chars.len();
                    if i + name_len > len {
                        return false;
                    }
                    let matches = (0..name_len).all(|j| chars[i + j] == name_chars[j]);
                    if !matches {
                        return false;
                    }
                    let before_ok =
                        i == 0 || !(chars[i - 1].is_alphabetic() || chars[i - 1] == '_');
                    let after_ok = i + name_len >= len
                        || !(chars[i + name_len].is_alphabetic() || chars[i + name_len] == '_');
                    before_ok && after_ok
                }) {
                    let color = match *symbol {
                        "/" | "\\" | "ǁ" => MAGENTA,
                        _ => CYAN,
                    };
                    // Show the unicode symbol, then pad with spaces to keep
                    // the same display width as the original alias text
                    out.push_str(color);
                    out.push_str(symbol);
                    out.push_str(RESET);
                    // Pad: alias was name.len() display columns wide,
                    // symbol is 1 display column wide
                    for _ in 1..name.len() {
                        out.push(' ');
                    }
                    i += name.len();
                    continue;
                }
            }

            // Comments: -- to end of line
            if ch == '-' && i + 1 < len && chars[i + 1] == '-' {
                out.push_str(GREY);
                while i < len {
                    out.push(chars[i]);
                    i += 1;
                }
                out.push_str(RESET);
                continue;
            }

            // Strings: "..." with "" as escape
            if ch == '"' {
                out.push_str(GREEN);
                out.push(ch);
                i += 1;
                while i < len {
                    out.push(chars[i]);
                    if chars[i] == '"' {
                        // check for escaped ""
                        if i + 1 < len && chars[i + 1] == '"' {
                            i += 1;
                            out.push(chars[i]);
                        } else {
                            break;
                        }
                    }
                    i += 1;
                }
                out.push_str(RESET);
                i += 1;
                continue;
            }

            // Numbers: digits, optional dot, more digits
            if ch.is_ascii_digit() {
                out.push_str(YELLOW);
                while i < len && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    out.push(chars[i]);
                    i += 1;
                }
                out.push_str(RESET);
                continue;
            }

            // Operators (including unicode ones)
            if matches!(ch, '+' | '×' | '÷' | '¯' | '=' | '&' | '!' | 'ρ') {
                out.push_str(CYAN);
                out.push(ch);
                out.push_str(RESET);
                i += 1;
                continue;
            }

            // - is an operator
            if ch == '-' {
                out.push_str(CYAN);
                out.push(ch);
                out.push_str(RESET);
                i += 1;
                continue;
            }

            // _ is min operator unless it's part of a variable name
            if ch == '_' {
                let in_var = i > 0 && (chars[i - 1].is_alphanumeric());
                if in_var {
                    out.push(ch);
                } else {
                    out.push_str(CYAN);
                    out.push(ch);
                    out.push_str(RESET);
                }
                i += 1;
                continue;
            }

            // Combinators
            if matches!(ch, '/' | '\\' | 'ǁ') {
                out.push_str(MAGENTA);
                out.push(ch);
                out.push_str(RESET);
                i += 1;
                continue;
            }

            // Pipes
            if ch == '|' {
                out.push_str(GREY);
                out.push(ch);
                out.push_str(RESET);
                i += 1;
                continue;
            }

            // Assignment
            if ch == ':' {
                out.push_str(BOLD);
                out.push(ch);
                out.push_str(RESET);
                i += 1;
                continue;
            }

            // Brackets
            if matches!(ch, '{' | '}' | '[' | ']' | '(' | ')') {
                out.push_str(BLUE);
                out.push(ch);
                out.push_str(RESET);
                i += 1;
                continue;
            }

            // Everything else (variables, whitespace)
            out.push(ch);
            i += 1;
        }

        out
    }
}

impl Highlighter for LinkHighlighter {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        Cow::Owned(self.highlight_line(line))
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _forced: bool) -> bool {
        true
    }
}

impl Completer for LinkHighlighter {
    type Candidate = String;
}

impl Hinter for LinkHighlighter {
    type Hint = String;
}

impl Validator for LinkHighlighter {}
impl Helper for LinkHighlighter {}

// ---------------------------------------------------------------------------
// REPL
// ---------------------------------------------------------------------------

fn main() {
    let mut rl = Editor::new().unwrap();
    rl.set_helper(Some(LinkHighlighter));
    println!("l prompt. Expressions are line evaluated.");
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(&line);
                let line = replace_aliases(&line);
                let byte_code = I::fstring(&line);

                match byte_code {
                    Ok(it) => {
                        let mut vm = V::new(it);
                        vm.r();
                        match &vm.error {
                            Some(e) => println!("{}", e),
                            None => match vm.pop_last() {
                                Some(result) => println!("{}", result),
                                None => println!("(no result)"),
                            },
                        }
                    }
                    Err(err) => println!("{}", err),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}
