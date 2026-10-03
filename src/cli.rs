use std::{ffi::OsStr, num::NonZero};

use clap::{ArgAction, Parser, ValueEnum};
use clap_complete::{ArgValueCompleter, CompletionCandidate, Shell};

use crate::{social_habit::SocialHabitFrequency, store::Store};

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

macro_rules! completer {
    ($name:ident, $items:ident) => {
        fn $name(current: &OsStr) -> Vec<CompletionCandidate> {
            let store = Store::new().unwrap();
            let start: &str = current.try_into().unwrap();
            store
                .$items
                .into_keys()
                .filter(|name| name.starts_with(start))
                .map(CompletionCandidate::new)
                .collect()
        }
    };
}

completer!(positive_habit_completer, positive_habits);

#[derive(clap::Subcommand)]
pub enum PositiveSubcommand {
    Add {
        name: String
    },
    Rename {
        #[arg(add = ArgValueCompleter::new(positive_habit_completer))]
        old: String,
        new: String
    },
    Remove {
        #[arg(add = ArgValueCompleter::new(positive_habit_completer))]
        name: String
    },
    Check {
        #[arg(add = ArgValueCompleter::new(positive_habit_completer))]
        name: String
    },
    Show {
        filter: Option<PositiveFilter>
    }
}

#[derive(ValueEnum, Clone, Copy)]
pub enum PositiveFilter {
    Done,
    NotDone
}

completer!(negative_habit_completer, negative_habits);

#[derive(clap::Subcommand)]
pub enum NegativeSubcommand {
    Add {
        name: String
    },
    Rename {
        #[arg(add = ArgValueCompleter::new(negative_habit_completer))]
        old: String,
        new: String
    },
    Remove {
        #[arg(add = ArgValueCompleter::new(negative_habit_completer))]
        name: String
    },
    Break {
        #[arg(add = ArgValueCompleter::new(negative_habit_completer))]
        name: String
    },
    Show
}

completer!(social_habit_completer, social_habits);

#[derive(clap::Subcommand)]
pub enum SocialSubcommand {
    Add {
        name: String,
        #[arg(long, short, default_value_t = SocialHabitFrequency::Medium)]
        frequency: SocialHabitFrequency
    },
    Rename {
        #[arg(add = ArgValueCompleter::new(social_habit_completer))]
        old: String,
        new: String
    },
    Remove {
        #[arg(add = ArgValueCompleter::new(social_habit_completer))]
        name: String
    },
    Check {
        #[arg(add = ArgValueCompleter::new(social_habit_completer))]
        name: String
    },
    SetFrequency {
        #[arg(add = ArgValueCompleter::new(social_habit_completer))]
        name: String,
        frequency: SocialHabitFrequency
    },
    Show {
        filter: Option<SocialFilter>
    }
}

#[derive(ValueEnum, Clone, Copy)]
pub enum SocialFilter {
    Pending,
    NotPending
}

completer!(task_completer, tasks);

#[derive(clap::Subcommand)]
pub enum TaskSubcommand {
    Add {
        name: String,
        #[arg(long, short, action = ArgAction::SetTrue)]
        queue: bool
    },
    Rename {
        #[arg(add = ArgValueCompleter::new(task_completer))]
        old: String,
        new: String
    },
    #[command(alias = "remove")]
    Complete {
        #[arg(add = ArgValueCompleter::new(task_completer))]
        name: String
    },
    Queue {
        #[arg(add = ArgValueCompleter::new(task_completer))]
        name: String
    },
    Unqueue {
        #[arg(add = ArgValueCompleter::new(task_completer))]
        name: String
    },
    Show {
        filter: Option<TaskFilter>
    }
}

#[derive(ValueEnum, Clone, Copy)]
pub enum TaskFilter {
    NotQueued,
    Queued
}
