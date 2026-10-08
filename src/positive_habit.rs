use std::cmp::Ordering;

use clap::ValueEnum;
use indexmap::IndexMap;
use jiff::Zoned;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use strum::AsRefStr;

use crate::{item::Item, store::Store, utils};

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
}

fn streak_text(streak: u32) -> String {
    let color = utils::days_to_color(streak);
    streak.color(color).bold().to_string()
}

impl Item for PositiveHabit {
    type Filter = Filter;
    fn filter(&self, filter: Filter, _now: &Zoned) -> bool {
        match filter {
            Filter::Done => self.done_today,
            Filter::NotDone => !self.done_today
        }
    }

    fn sort(&self, other: &Self, _now: &Zoned) -> Ordering {
        self.done_today.cmp(&other.done_today)
    }

    fn get_items_mut(store: &mut Store) -> &mut IndexMap<String, Self> {
        &mut store.positive_habits
    }

    fn show(&self, name: &str, filter: Option<Filter>, _now: &Zoned) {
        let spacer = " • ".bright_black();
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
        let pending = if !self.done_today && filter.is_none() {
            "[Pending]".yellow().to_string()
        } else {
            String::new()
        };
        println!("{pending} {name}{spacer}Streak: {streak_text}{record}");
    }
}

#[derive(ValueEnum, Clone, Copy, AsRefStr)]
pub enum Filter {
    #[strum(serialize = "Must be done!")]
    Done,
    #[strum(serialize = "Must not be done!")]
    NotDone
}
