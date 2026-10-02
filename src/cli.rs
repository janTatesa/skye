use std::num::NonZero;

use clap::{ArgAction, Parser, ValueEnum};
use clap_complete::Shell;

use crate::social_habit::SocialHabitFrequency;

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

#[derive(clap::Subcommand)]
pub enum PositiveSubcommand {
    Add { name: String },
    Rename { old: String, new: String },
    Remove { name: Option<String> },
    Check { name: Option<String> },
    Show { filter: Option<PositiveFilter> }
}

#[derive(ValueEnum, Clone, Copy)]
pub enum PositiveFilter {
    Done,
    NotDone
}

#[derive(clap::Subcommand)]
pub enum NegativeSubcommand {
    Add { name: String },
    Rename { old: String, new: String },
    Remove { name: Option<String> },
    Break { name: Option<String> },
    Show
}

#[derive(clap::Subcommand)]
pub enum SocialSubcommand {
    Add {
        name: String,
        frequency: SocialHabitFrequency
    },
    Rename {
        old: String,
        new: String
    },
    Remove {
        name: Option<String>
    },
    AddInteraction {
        name: Option<String>
    },
    SetFrequency {
        name: Option<String>,
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

#[derive(clap::Subcommand)]
pub enum TaskSubcommand {
    Add {
        name: String,
        #[arg(long, short, action = ArgAction::SetTrue)]
        queue: bool
    },
    Rename {
        old: String,
        new: String
    },
    #[command(alias = "remove")]
    Complete {
        name: Option<String>
    },
    Queue {
        name: Option<String>
    },
    Unqueue {
        name: Option<String>
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
