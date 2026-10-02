use owo_colors::{OwoColorize, Style};
use serde::{Deserialize, Serialize};

use crate::utils;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PositiveHabit {
    streak: u32,
    done_today: bool,
    record: Option<u32>
}

impl PositiveHabit {
    pub fn reset(&mut self) {
        self.done_today = false;
        if self.streak > 0 {
            self.record = self.record.max(Some(self.streak));
        }
        self.streak = 0;
    }

    pub fn on_next_day(&mut self) {
        if !self.done_today && self.streak > 0 {
            self.record = self.record.max(Some(self.streak));
            self.streak = 0;
        } else {
            self.done_today = false;
        }
    }

    pub fn check(&mut self) {
        self.streak += 1;
        self.done_today = true;
        let record_msg = if self.record.is_some_and(|record| record < self.streak) {
            "That is a new record!"
        } else {
            ""
        };
        let record_msg = record_msg.green();
        let record_msg = record_msg.bold();
        println!(
            "Congrats, You done it for today! The streak is now {}! {record_msg}",
            streak_text(self.streak),
        );
    }

    pub fn done_today(&self) -> bool {
        self.done_today
    }

    pub fn display(&self, name: &str) {
        let spacer = " • ".bright_black();

        let name = name.style(if self.done_today() {
            Style::new().italic()
        } else {
            Style::new().italic().bold().yellow()
        });
        let name = name.italic();

        let record = self
            .record
            .map(|record| {
                if self.streak > record {
                    format!("{spacer}{}", "That's a new record!".green().bold())
                } else {
                    format!("{spacer}Record: {}", streak_text(self.streak))
                }
            })
            .unwrap_or_default();
        let streak_text = streak_text(self.streak);
        println!("{name}{spacer}Streak: {streak_text}{record}");
    }
}

fn streak_text(streak: u32) -> String {
    let color = utils::days_to_color(streak);
    streak.color(color).bold().to_string()
}
