#![deny(clippy::all, clippy::pedantic)]
#![allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]

mod cli;
mod daemon;
mod item;
mod store;
mod utils;

mod negative_habit;
mod positive_habit;
mod social_habit;
mod task;

use std::time::Duration;

use clap::{CommandFactory, Parser};
use clap_complete::{CompleteEnv, Shell};
use jiff::Zoned;

use crate::{
    cli::{
        Cli, NegativeSubcommand, PositiveSubcommand, SocialSubcommand, Subcommand, TaskSubcommand
    },
    item::{add, get_mut, remove, rename, show},
    negative_habit::NegativeHabit,
    positive_habit::PositiveHabit,
    social_habit::SocialHabit,
    store::Store,
    task::Task,
    utils::exit_with_error
};

fn main() -> color_eyre::Result {
    color_eyre::install()?;
    CompleteEnv::with_factory(Cli::command).complete();
    let cli = Cli::parse();
    let store = &mut Store::new()?;
    let now = &Zoned::now();
    match cli.subcommand {
        Subcommand::Completions { shell } => {
            let completions = match shell {
                Shell::Bash => "source <(COMPLETE=bash skye)",
                Shell::Elvish => "eval (E:COMPLETE=elvish skye | slurp)",
                Shell::Fish => "COMPLETE=fish your_program | source",
                Shell::PowerShell => {
                    "$env:COMPLETE = \"powershell\"; your_program | Out-String | Invoke-Expression; Remove-Item Env:\\COMPLETE"
                }
                Shell::Zsh => "source <(COMPLETE=zsh your_program)",
                _ => exit_with_error("Unsupported shell")
            };
            println!("{completions}");
        }
        Subcommand::Daemon {
            mins_between_reminders
        } => {
            return daemon::daemon(store, Duration::from_mins(mins_between_reminders.get()));
        }
        Subcommand::Positive {
            subcommand: PositiveSubcommand::Add { name }
        } => add(store, name, PositiveHabit::default()),
        Subcommand::Positive {
            subcommand: PositiveSubcommand::Remove { name }
        } => remove::<PositiveHabit>(store, &name),
        Subcommand::Positive {
            subcommand: PositiveSubcommand::Check { name }
        } => get_mut::<PositiveHabit>(store, &name, Some(positive_habit::Filter::NotDone), now)
            .check(),
        Subcommand::Positive {
            subcommand: PositiveSubcommand::Rename { old, new }
        } => rename::<PositiveHabit>(store, &old, new),
        Subcommand::Positive {
            subcommand: PositiveSubcommand::Show { filter }
        } => show::<PositiveHabit>(store, filter, now),
        Subcommand::Negative {
            subcommand: NegativeSubcommand::Add { name }
        } => add(store, name, NegativeHabit::default()),
        Subcommand::Negative {
            subcommand: NegativeSubcommand::Rename { old, new }
        } => rename::<NegativeHabit>(store, &old, new),
        Subcommand::Negative {
            subcommand: NegativeSubcommand::Break { name }
        } => get_mut::<NegativeHabit>(store, &name, None, now).r#break(now),
        Subcommand::Negative {
            subcommand: NegativeSubcommand::Remove { name }
        } => remove::<NegativeHabit>(store, &name),
        Subcommand::Negative {
            subcommand: NegativeSubcommand::Show
        } => show::<NegativeHabit>(store, None, now),
        Subcommand::Social {
            subcommand: SocialSubcommand::Add { name, frequency }
        } => add(store, name, SocialHabit::new(frequency)),
        Subcommand::Social {
            subcommand: SocialSubcommand::Rename { old, new }
        } => rename::<SocialHabit>(store, &old, new),
        Subcommand::Social {
            subcommand: SocialSubcommand::Remove { name }
        } => remove::<SocialHabit>(store, &name),
        Subcommand::Social {
            subcommand: SocialSubcommand::SetFrequency { name, frequency }
        } => get_mut::<SocialHabit>(store, &name, None, now).frequency = frequency,
        Subcommand::Social {
            subcommand: SocialSubcommand::Check { name }
        } => get_mut::<SocialHabit>(store, &name, None, now).check(now),
        Subcommand::Social {
            subcommand: SocialSubcommand::Show { filter }
        } => show::<SocialHabit>(store, filter, now),
        Subcommand::Task {
            subcommand: TaskSubcommand::Add { name, queued }
        } => add(store, name, Task { queued }),
        Subcommand::Task {
            subcommand: TaskSubcommand::Rename { old, new }
        } => rename::<Task>(store, &old, new),
        Subcommand::Task {
            subcommand: TaskSubcommand::Complete { name }
        } => remove::<Task>(store, &name),
        Subcommand::Task {
            subcommand: TaskSubcommand::Queue { name }
        } => get_mut::<Task>(store, &name, Some(task::Filter::NotQueued), now).queued = true,
        Subcommand::Task {
            subcommand: TaskSubcommand::Unqueue { name }
        } => get_mut::<Task>(store, &name, Some(task::Filter::Queued), now).queued = false,
        Subcommand::Task {
            subcommand: TaskSubcommand::Show { filter }
        } => show::<Task>(store, filter, now)
    }

    store.save()
}
