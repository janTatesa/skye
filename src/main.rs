#![deny(clippy::all, clippy::pedantic)]
#![expect(clippy::too_many_lines)]
#![allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]

mod cli;
mod daemon;
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
        Cli, NegativeSubcommand, PositiveFilter, PositiveSubcommand, SocialFilter,
        SocialSubcommand, Subcommand, TaskFilter, TaskSubcommand
    },
    negative_habit::NegativeHabit,
    positive_habit::PositiveHabit,
    social_habit::SocialHabit,
    store::Store,
    task::Task,
    utils::{GetMutFilter, add, exit_with_error, get_mut, remove, rename}
};

fn main() -> color_eyre::Result {
    color_eyre::install()?;
    CompleteEnv::with_factory(Cli::command).complete();
    let cli = Cli::parse();
    let mut store = Store::new()?;
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
        } => add(&mut store.positive_habits, name, PositiveHabit::default()),
        Subcommand::Positive {
            subcommand: PositiveSubcommand::Remove { name }
        } => remove(&mut store.positive_habits, &name),
        Subcommand::Positive {
            subcommand: PositiveSubcommand::Check { name }
        } => get_mut(
            &mut store.positive_habits,
            &name,
            Some(&GetMutFilter {
                function: |habit| !habit.done_today(),
                error: "Has already been checked"
            })
        )
        .check(),
        Subcommand::Positive {
            subcommand: PositiveSubcommand::Rename { old, new }
        } => rename(&mut store.positive_habits, &old, new),
        Subcommand::Positive {
            subcommand: PositiveSubcommand::Show { filter }
        } => {
            for (name, habit) in &store.positive_habits {
                match filter {
                    Some(PositiveFilter::Done) if !habit.done_today() => {}
                    Some(PositiveFilter::NotDone) if habit.done_today() => {}
                    _ => habit.display(name)
                }
            }
        }
        Subcommand::Negative {
            subcommand: NegativeSubcommand::Add { name }
        } => add(&mut store.negative_habits, name, NegativeHabit::default()),
        Subcommand::Negative {
            subcommand: NegativeSubcommand::Rename { old, new }
        } => rename(&mut store.negative_habits, &old, new),
        Subcommand::Negative {
            subcommand: NegativeSubcommand::Break { name }
        } => get_mut(&mut store.negative_habits, &name, None).r#break(&Zoned::now()),
        Subcommand::Negative {
            subcommand: NegativeSubcommand::Remove { name }
        } => {
            remove(&mut store.negative_habits, &name);
        }
        Subcommand::Negative {
            subcommand: NegativeSubcommand::Show
        } => {
            let now = &Zoned::now();
            for (name, habit) in &store.negative_habits {
                habit.display(name, now);
            }
        }
        Subcommand::Social {
            subcommand: SocialSubcommand::Add { name, frequency }
        } => add(&mut store.social_habits, name, SocialHabit::new(frequency)),
        Subcommand::Social {
            subcommand: SocialSubcommand::Rename { old, new }
        } => rename(&mut store.social_habits, &old, new),
        Subcommand::Social {
            subcommand: SocialSubcommand::Remove { name }
        } => remove(&mut store.social_habits, &name),
        Subcommand::Social {
            subcommand: SocialSubcommand::SetFrequency { name, frequency }
        } => get_mut(&mut store.social_habits, &name, None).frequency = frequency,
        Subcommand::Social {
            subcommand: SocialSubcommand::Check { name }
        } => get_mut(&mut store.social_habits, &name, None).check(&Zoned::now()),
        Subcommand::Social {
            subcommand: SocialSubcommand::Show { filter }
        } => {
            let now = &Zoned::now();
            for (name, habit) in &store.social_habits {
                match filter {
                    Some(SocialFilter::Pending) if !habit.pending(now) => {}
                    Some(SocialFilter::NotPending) if habit.pending(now) => {}
                    _ => habit.display(name, now)
                }
            }
        }
        Subcommand::Task {
            subcommand: TaskSubcommand::Add { name, queue }
        } => add(&mut store.tasks, name, Task { queued: queue }),
        Subcommand::Task {
            subcommand: TaskSubcommand::Rename { old, new }
        } => rename(&mut store.tasks, &old, new),
        Subcommand::Task {
            subcommand: TaskSubcommand::Complete { name }
        } => remove(&mut store.tasks, &name),
        Subcommand::Task {
            subcommand: TaskSubcommand::Queue { name }
        } => {
            get_mut(
                &mut store.tasks,
                &name,
                Some(&GetMutFilter {
                    function: |task| !task.queued,
                    error: "Task is already queued"
                })
            )
            .queued = true;
        }
        Subcommand::Task {
            subcommand: TaskSubcommand::Unqueue { name }
        } => {
            get_mut(
                &mut store.tasks,
                &name,
                Some(&GetMutFilter {
                    function: |task| task.queued,
                    error: "Task is not queued"
                })
            )
            .queued = false;
        }
        Subcommand::Task {
            subcommand: TaskSubcommand::Show { filter }
        } => {
            for (name, task) in &store.tasks {
                match filter {
                    Some(TaskFilter::NotQueued) if task.queued => {}
                    Some(TaskFilter::Queued) if !task.queued => {}
                    _ => task.display(name)
                }
            }
        }
    }

    store.save()
}
