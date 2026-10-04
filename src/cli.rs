use std::{ffi::OsStr, num::NonZero};

use clap::{ArgAction, Parser, builder::StyledStr};
use clap_complete::{ArgValueCompleter, CompletionCandidate, Shell, engine::ValueCompleter};
use jiff::Zoned;

use crate::{
    item::Item,
    negative_habit::NegativeHabit,
    positive_habit::{self, PositiveHabit},
    social_habit::{self, Frequency, SocialHabit},
    store::Store,
    task::{self, Task}
};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: Subcommand
}

#[derive(clap::Subcommand)]
pub enum Subcommand {
    /// Generate completions
    Completions { shell: Shell },
    /// Run the notification daemon
    Daemon {
        #[arg(long, short, default_value_t = NonZero::new(30).unwrap())]
        mins_between_reminders: NonZero<u64>
    },
    /// Manage positive habits
    Positive {
        #[command(subcommand)]
        subcommand: PositiveSubcommand
    },
    /// Manage negative habits
    Negative {
        #[command(subcommand)]
        subcommand: NegativeSubcommand
    },
    /// Manage social habits
    Social {
        #[command(subcommand)]
        subcommand: SocialSubcommand
    },
    /// Manage tasks
    Task {
        #[command(subcommand)]
        subcommand: TaskSubcommand
    }
}

struct Completer<T: Item> {
    filter: Option<T::Filter>
}

impl<T: Item> ValueCompleter for Completer<T> {
    fn complete(&self, current: &OsStr) -> Vec<CompletionCandidate> {
        let now = &Zoned::now();
        let mut store = Store::new().unwrap();
        let start: &str = current.try_into().unwrap();
        let mut items: Vec<_> = T::get_items_mut(&mut store)
            .iter()
            .filter(|(name, val)| {
                name.starts_with(start) && self.filter.is_none_or(|filter| val.filter(filter, now))
            })
            .collect();
        items.sort_by(|(_, a), (_, b)| b.sort(a, now));
        items
            .into_iter()
            .map(|(name, item)| {
                CompletionCandidate::new(name)
                    .help(item.completion_help(self.filter, now).map(StyledStr::from))
            })
            .collect()
    }
}

#[derive(clap::Subcommand)]
pub enum PositiveSubcommand {
    Add {
        name: String
    },
    Rename {
        #[arg(add = ArgValueCompleter::new(Completer::<PositiveHabit> { filter: None }))]
        old: String,
        new: String
    },
    Remove {
        #[arg(add = ArgValueCompleter::new(Completer::<PositiveHabit> { filter: None }))]
        name: String
    },
    Check {
        #[arg(add = ArgValueCompleter::new(Completer::<PositiveHabit> { filter: Some(positive_habit::Filter::NotDone) }))]
        name: String
    },
    Show {
        filter: Option<positive_habit::Filter>
    }
}

#[derive(clap::Subcommand)]
pub enum NegativeSubcommand {
    Add {
        name: String
    },
    Rename {
        #[arg(add = ArgValueCompleter::new(Completer::<NegativeHabit> { filter: None }))]
        old: String,
        new: String
    },
    Remove {
        #[arg(add = ArgValueCompleter::new(Completer::<NegativeHabit> { filter: None }))]
        name: String
    },
    Break {
        #[arg(add = ArgValueCompleter::new(Completer::<NegativeHabit> { filter: None }))]
        name: String
    },
    Show
}

#[derive(clap::Subcommand)]
pub enum SocialSubcommand {
    Add {
        name: String,
        #[arg(long, short, default_value = "medium")]
        frequency: Frequency
    },
    Rename {
        #[arg(add = ArgValueCompleter::new(Completer::<SocialHabit> { filter: None }))]
        old: String,
        new: String
    },
    Remove {
        #[arg(add = ArgValueCompleter::new(Completer::<SocialHabit> { filter: None }))]
        name: String
    },
    Check {
        #[arg(add = ArgValueCompleter::new(Completer::<SocialHabit> { filter: None }))]
        name: String
    },
    SetFrequency {
        #[arg(add = ArgValueCompleter::new(Completer::<SocialHabit> { filter: None }))]
        name: String,
        frequency: Frequency
    },
    Show {
        filter: Option<social_habit::Filter>
    }
}

#[derive(clap::Subcommand)]
pub enum TaskSubcommand {
    Add {
        name: String,
        #[arg(long, short, action = ArgAction::SetTrue)]
        queued: bool
    },
    Rename {
        #[arg(add = ArgValueCompleter::new(Completer::<Task> { filter: None }))]
        old: String,
        new: String
    },
    #[command(alias = "remove")]
    Complete {
        #[arg(add = ArgValueCompleter::new(Completer::<Task> { filter: None }))]
        name: String
    },
    Queue {
        #[arg(add = ArgValueCompleter::new(Completer::<Task> { filter: Some(task::Filter::NotQueued) }))]
        name: String
    },
    Unqueue {
        #[arg(add = ArgValueCompleter::new(Completer::<Task> { filter: Some(task::Filter::Queued) } ))]
        name: String
    },
    Show {
        filter: Option<task::Filter>
    }
}
