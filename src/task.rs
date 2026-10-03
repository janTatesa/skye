use std::cmp::Ordering;

use clap::ValueEnum;
use indexmap::IndexMap;
use jiff::Zoned;
use owo_colors::{OwoColorize, Style};
use serde::{Deserialize, Serialize};
use strum::AsRefStr;

use crate::{item::Item, store::Store};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Task {
    pub queued: bool
}

impl Item for Task {
    type Filter = Filter;

    fn get_items_mut(store: &mut Store) -> &mut IndexMap<String, Self> {
        &mut store.tasks
    }

    fn show(&self, name: &str, _now: &Zoned) {
        let style = if self.queued {
            Style::new().bold().yellow().italic()
        } else {
            Style::new().italic()
        };

        println!("{}", name.style(style));
    }

    fn filter(&self, filter: Filter, _now: &Zoned) -> bool {
        match filter {
            Filter::NotQueued => !self.queued,
            Filter::Queued => self.queued
        }
    }

    fn sort(&self, other: &Self, _now: &Zoned) -> Ordering {
        self.queued.cmp(&other.queued)
    }

    fn completion_help(&self, filter: Option<Self::Filter>, _now: &Zoned) -> Option<String> {
        if filter.is_none() && self.queued {
            Some("Queued".to_string())
        } else {
            None
        }
    }
}

#[derive(ValueEnum, Clone, Copy, AsRefStr)]
pub enum Filter {
    #[strum(serialize = "Must not be queued!")]
    NotQueued,
    #[strum(serialize = "Must be queued!")]
    Queued
}
