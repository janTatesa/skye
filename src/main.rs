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

use std::{io::Write, os::unix::net::UnixStream, time::Duration};

use clap::{CommandFactory, Parser};
use clap_complete::{CompleteEnv, Shell};
use jiff::Zoned;

use crate::{
    cli::{
        Cli, NegativeSubcommand, PositiveSubcommand, SocialSubcommand, Subcommand, TaskSubcommand
    },
    daemon::{Daemon, SOCKET_PATH},
    item::{add, get_mut, remove, rename, show, show_all},
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
    let mut store = Store::new()?;
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
            return Daemon::run(store, Duration::from_mins(mins_between_reminders.get()));
        }
        Subcommand::Positive {
            subcommand: PositiveSubcommand::Add { name }
        } => add(&mut store, name, PositiveHabit::default()),
        Subcommand::Positive {
            subcommand: PositiveSubcommand::Remove { name }
        } => remove::<PositiveHabit>(&mut store, &name),
        Subcommand::Positive {
            subcommand: PositiveSubcommand::Check { name }
        } => {
            let filter = Some(positive_habit::Filter::NotDone);
            get_mut::<PositiveHabit>(&mut store, &name, filter, now).check();
        }
        Subcommand::Positive {
            subcommand: PositiveSubcommand::Rename { old, new }
        } => rename::<PositiveHabit>(&mut store, &old, new),
        Subcommand::Positive {
            subcommand: PositiveSubcommand::Show { filter }
        } => show::<PositiveHabit>(&mut store, filter, now),
        Subcommand::Negative {
            subcommand: NegativeSubcommand::Add { name }
        } => add(&mut store, name, NegativeHabit::default()),
        Subcommand::Negative {
            subcommand: NegativeSubcommand::Rename { old, new }
        } => rename::<NegativeHabit>(&mut store, &old, new),
        Subcommand::Negative {
            subcommand: NegativeSubcommand::Break { name }
        } => get_mut::<NegativeHabit>(&mut store, &name, None, now).r#break(now),
        Subcommand::Negative {
            subcommand: NegativeSubcommand::Remove { name }
        } => remove::<NegativeHabit>(&mut store, &name),
        Subcommand::Negative {
            subcommand: NegativeSubcommand::Show
        } => show::<NegativeHabit>(&mut store, None, now),
        Subcommand::Social {
            subcommand: SocialSubcommand::Add { name, frequency }
        } => add(&mut store, name, SocialHabit::new(frequency)),
        Subcommand::Social {
            subcommand: SocialSubcommand::Rename { old, new }
        } => rename::<SocialHabit>(&mut store, &old, new),
        Subcommand::Social {
            subcommand: SocialSubcommand::Remove { name }
        } => remove::<SocialHabit>(&mut store, &name),
        Subcommand::Social {
            subcommand: SocialSubcommand::SetFrequency { name, frequency }
        } => get_mut::<SocialHabit>(&mut store, &name, None, now).frequency = frequency,
        Subcommand::Social {
            subcommand: SocialSubcommand::Check { name }
        } => get_mut::<SocialHabit>(&mut store, &name, None, now).check(now),
        Subcommand::Social {
            subcommand: SocialSubcommand::Show { filter }
        } => show::<SocialHabit>(&mut store, filter, now),
        Subcommand::Task {
            subcommand: TaskSubcommand::Add { name, queued }
        } => add(&mut store, name, Task { queued }),
        Subcommand::Task {
            subcommand: TaskSubcommand::Rename { old, new }
        } => rename::<Task>(&mut store, &old, new),
        Subcommand::Task {
            subcommand: TaskSubcommand::Complete { name }
        } => remove::<Task>(&mut store, &name),
        Subcommand::Task {
            subcommand: TaskSubcommand::Queue { name }
        } => get_mut::<Task>(&mut store, &name, Some(task::Filter::NotQueued), now).queued = true,
        Subcommand::Task {
            subcommand: TaskSubcommand::Unqueue { name }
        } => get_mut::<Task>(&mut store, &name, Some(task::Filter::Queued), now).queued = false,
        Subcommand::Task {
            subcommand: TaskSubcommand::Show { filter }
        } => show::<Task>(&mut store, filter, now),
        Subcommand::ShowAll { pending } => show_all(&mut store, now, pending)
    }

    if SOCKET_PATH.exists() {
        let mut stream = UnixStream::connect(&*SOCKET_PATH)?;
        stream.write_all(serde_json::to_string(&store)?.as_bytes())?;
    }

    store.save()
}
