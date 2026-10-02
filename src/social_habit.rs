use clap::ValueEnum;
use jiff::{Zoned, civil::Date};
use owo_colors::{AnsiColors, OwoColorize, Style};
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Serialize, Deserialize)]
pub struct SocialHabit {
    last_interaction: Option<Date>,
    pub frequency: SocialHabitFrequency
}

impl SocialHabit {
    pub fn new(frequency: SocialHabitFrequency) -> Self {
        Self {
            last_interaction: None,
            frequency
        }
    }

    pub fn last_interaction(&self) -> Option<Date> {
        self.last_interaction
    }

    pub fn check(&mut self, now: &Zoned) {
        self.last_interaction = Some(now.date());
    }

    pub fn pending(&self, now: &Zoned) -> bool {
        self.last_interaction().is_none_or(|date| {
            date.duration_until(now.date()).as_hours() / 24 > self.frequency.days()
        })
    }

    pub fn display(&self, name: &str, now: &Zoned) {
        let name = name.italic();
        let spacer = " • ".bright_black();
        let name = name.style(if self.pending(now) {
            Style::new().italic().bold().yellow()
        } else {
            Style::new().italic()
        });
        let frequency = self.frequency.color(match self.frequency {
            SocialHabitFrequency::Low => AnsiColors::Green,
            SocialHabitFrequency::Medium => AnsiColors::Yellow,
            SocialHabitFrequency::High => AnsiColors::Red
        });
        let last_interaction = self
            .last_interaction
            .map_or_default(|date| date.blue().underline().to_string());
        println!("{name}{spacer}Frequency: {frequency}{spacer}{last_interaction}");
    }
}

#[derive(Debug, Serialize, Deserialize, Display, Clone, Copy, ValueEnum)]
pub enum SocialHabitFrequency {
    Low,
    Medium,
    High
}

impl SocialHabitFrequency {
    pub fn days(self) -> i64 {
        match self {
            SocialHabitFrequency::Low => 30,
            SocialHabitFrequency::Medium => 7,
            SocialHabitFrequency::High => 2
        }
    }
}
