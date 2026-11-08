//! Printing: translated messages, colours on a terminal, and asking before sudo.

use cuvee_core::i18n::Locale;
use std::io::IsTerminal;
use std::sync::OnceLock;

/// A message argument: a number or a string.
pub enum Arg<'a> {
    N(f64),
    S(&'a str),
}

fn locale() -> &'static Locale {
    static L: OnceLock<Locale> = OnceLock::new();
    L.get_or_init(|| {
        let s = cuvee_core::settings::Settings::load();
        let tag = cuvee_core::i18n::system_language().unwrap_or(&s.language);
        Locale::new(tag)
    })
}

/// The message `key` with its arguments.
pub fn t(key: &str, args: &[(&str, Arg)]) -> String {
    let a: Vec<_> = args
        .iter()
        .map(|(k, v)| {
            (
                *k,
                match v {
                    Arg::N(n) => Locale::num(*n),
                    Arg::S(s) => Locale::str(s),
                },
            )
        })
        .collect();
    locale().t_args(key, &a)
}

fn color() -> bool {
    std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none()
}

fn paint(code: &str, s: &str) -> String {
    if color() {
        format!("\x1b[{code}m{s}\x1b[0m")
    } else {
        s.to_string()
    }
}

pub fn title(s: &str) {
    println!("\n{}", paint("1", s));
}

pub fn ok(s: &str) {
    println!("  {} {s}", paint("32", "✓"));
}

pub fn warn(s: &str) {
    println!("  {} {s}", paint("33", "!"));
}

pub fn bad(s: &str) {
    println!("  {} {s}", paint("31", "✗"));
}

pub fn info(s: &str) {
    println!("  {s}");
}

pub fn error(s: &str) {
    let tag = if std::io::stderr().is_terminal() {
        "\x1b[31mcuvee:\x1b[0m"
    } else {
        "cuvee:"
    };
    eprintln!("{tag} {s}");
}

pub fn red(s: &str) -> String {
    paint("31", s)
}

pub fn green(s: &str) -> String {
    paint("32", s)
}
