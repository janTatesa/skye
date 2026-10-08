use clap::ValueEnum;
use jiff::{Zoned, civil::Date};
use owo_colors::{AnsiColors, OwoColorize};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display};

use crate::{item::Item, store::Store};

#[derive(Debug, Serialize, Deserialize)]
pub struct SocialHabit {
    last_interaction: Option<Date>,
    pub frequency: Frequency
}

impl SocialHabit {
    pub fn new(frequency: Frequency) -> Self {
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

    pub fn days_pending(&self, now: &Zoned) -> Option<u32> {
        self.last_interaction().map(|date| {
            (date.duration_until(now.date()).as_hours() / 24 - self.frequency.days()).max(0) as u32
        })
    }
}

#[derive(
    Debug, Serialize, Deserialize, Display, Clone, Copy, ValueEnum, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum Frequency {
    Low,
    Medium,
    High
}

impl Frequency {
    pub fn days(self) -> i64 {
        match self {
            Frequency::Low => 30,
            Frequency::Medium => 7,
            Frequency::High => 2
        }
    }
}

impl Item for SocialHabit {
    type Filter = Filter;

    fn get_items_mut(store: &mut Store) -> &mut indexmap::IndexMap<String, Self> {
        &mut store.social_habits
    }

    fn show(&self, name: &str, filter: Option<Filter>, now: &Zoned) {
        let spacer = " • ".bright_black();
        let frequency = self.frequency.color(match self.frequency {
            Frequency::Low => AnsiColors::Green,
            Frequency::Medium => AnsiColors::Yellow,
            Frequency::High => AnsiColors::Red
        });
        let last_interaction = self
            .last_interaction
            .map_or_default(|date| format!("{spacer}{}", date.blue().underline()));
        let pending = if self.days_pending(now).is_none_or(|days| days > 0) && filter.is_none() {
            "[Pending]".yellow().to_string()
        } else {
            String::new()
        };

        println!("{pending} {name}{spacer}Frequency: {frequency}{last_interaction}");
    }

    fn filter(&self, filter: Self::Filter, now: &Zoned) -> bool {
        match filter {
            Filter::Pending => self.days_pending(now).is_none_or(|days| days > 0),
            Filter::NotPending => self.days_pending(now) == Some(0)
        }
    }

    fn sort(&self, other: &Self, now: &Zoned) -> std::cmp::Ordering {
        self.days_pending(now)
            .unwrap_or(u32::MAX)
            .cmp(&other.days_pending(now).unwrap_or(u32::MAX))
            .then(self.frequency.cmp(&other.frequency))
    }

    fn completion_help(&self, filter: Option<Self::Filter>, now: &Zoned) -> Option<String> {
        if filter.is_none() && self.days_pending(now).is_none_or(|days| days > 0) {
            Some(String::from("Pending"))
        } else {
            None
        }
    }
}

#[derive(ValueEnum, Clone, Copy, AsRefStr)]
pub enum Filter {
    #[strum(serialize = "Must be pending!")]
    Pending,
    #[strum(serialize = "Must not be pending!")]
    NotPending
}
