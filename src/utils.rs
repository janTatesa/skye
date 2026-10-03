use owo_colors::{AnsiColors, OwoColorize};

pub fn exit_with_error(error: &str) -> ! {
    eprintln!("{}", error.red());
    std::process::exit(1)
}

#[expect(clippy::match_overlapping_arm, reason = "pwettyness")]
pub fn days_to_color(days: u32) -> AnsiColors {
    match days {
        ..1 => AnsiColors::Red,
        ..3 => AnsiColors::Yellow,
        ..7 => AnsiColors::Green,
        ..14 => AnsiColors::Cyan,
        ..21 => AnsiColors::Blue,
        _ => AnsiColors::Magenta
    }
}
