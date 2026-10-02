use std::time::{Duration, Instant};

use futures::{
    FutureExt as _, SinkExt,
    channel::mpsc::{self, UnboundedReceiver},
    future::pending
};
use indexmap::IndexMap;
use jiff::{Timestamp, Zoned, civil::Time};
use notify::Watcher;
use notify_rust::{Notification, Urgency};
use smol::future::FutureExt as _;

use crate::{
    negative_habit::{Milestone, NegativeHabit},
    store::Store
};

enum DaemonCommand<'a> {
    ReloadStore,
    Milestone {
        milestone: Milestone,
        habit: &'a mut NegativeHabit,
        name: &'a str
    },
    Reminder
}

pub fn daemon(mut store: Store, remider_duration: Duration) -> color_eyre::Result {
    let (store_reload_tx, mut store_reload_rx) = mpsc::unbounded();
    let mut watcher = notify::recommended_watcher(move |event| match event {
        Ok(notify::Event {
            kind: notify::EventKind::Any | notify::EventKind::Modify(_) | notify::EventKind::Other,
            ..
        }) => _ = smol::block_on(store_reload_tx.clone().send(())),
        Ok(_) => {}
        Err(error) => log::error!("Watcher error: {error}")
    })?;
    watcher.watch(&crate::store::PATH, notify::RecursiveMode::NonRecursive)?;

    let mut last_reminder = None;
    let mut empty_reminders = 0;
    loop {
        if let Err(error) = iterate(
            &mut store,
            &mut store_reload_rx,
            &mut last_reminder,
            remider_duration,
            &mut empty_reminders
        ) {
            log::error!("Daemon error: {error}");
            notification("Daemon error", &error.to_string())?;
        }
    }
}

fn iterate(
    store: &mut Store,
    store_reload_rx: &mut UnboundedReceiver<()>,
    last_reminder: &mut Option<Instant>,
    remider_duration: Duration,
    empty_reminders: &mut u64
) -> color_eyre::Result {
    let now = &Zoned::now();
    let reload = store_reload_rx.recv().map(|_| DaemonCommand::ReloadStore);
    let reminder = smol::Timer::at(last_reminder.map_or(Instant::now(), |last_reminder| {
        last_reminder + remider_duration + Duration::from_secs(*empty_reminders)
    }))
    .map(|_| DaemonCommand::Reminder);
    let duration_till_next_day = now
        .datetime()
        .duration_until(now.date().tomorrow().unwrap().to_datetime(Time::midnight()))
        .try_into()?;
    let next_day = smol::Timer::after(duration_till_next_day).map(|_| DaemonCommand::ReloadStore);
    let milestone = async {
        match store
            .negative_habits
            .iter_mut()
            .map(|(name, habit)| (name, habit.next_milestone(now), habit))
            .min_by_key(|(_, (_, timestamp), _)| *timestamp)
        {
            Some((name, (milestone, finish), habit)) => {
                let duration = finish
                    .duration_until(Timestamp::now())
                    .try_into()
                    .unwrap_or_default();
                smol::Timer::after(duration)
                    .map(|_| DaemonCommand::Milestone {
                        milestone,
                        habit,
                        name: name.as_str()
                    })
                    .await
            }
            None => pending().await
        }
    };
    let command = smol::block_on(reload.or(reminder).or(milestone).or(next_day));
    match command {
        DaemonCommand::ReloadStore => {
            log::info!("Reloading");
            *store = Store::new()?;
        }
        DaemonCommand::Milestone {
            milestone,
            habit,
            name
        } => {
            let notification_body = format!(
                "You reached a {milestone} milestone without the bad habit of <i>{name}</i>. Congrats!",
            );
            notification("Milestone", &notification_body)?;

            habit.last_milestone = Some(milestone);
            store.save()?;
        }
        DaemonCommand::Reminder => {
            fn section<T>(
                title: &str,
                store: &IndexMap<String, T>,
                filter: impl Fn(&T) -> bool,
                body: &mut String
            ) -> bool {
                let items: Vec<_> = store
                    .iter()
                    .filter(|(_, item)| filter(*item))
                    .flat_map(|(item, _)| ["\n - ", item])
                    .collect();
                if items.is_empty() {
                    return false;
                }

                body.push_str(title);
                body.extend(items);

                true
            }

            let mut body = String::new();

            let habits = section(
                "<b>Habits:</b>",
                &store.positive_habits,
                |habit| !habit.done_today(),
                &mut body
            );
            let social = section(
                "\n<b>Social:</b>",
                &store.social_habits,
                |habit| habit.pending(now),
                &mut body
            );
            let tasks = section(
                "\n<b>Tasks:</b>",
                &store.tasks,
                |task| task.queued,
                &mut body
            );
            if habits || social || tasks {
                notification("Reminder", &body)?;
                *empty_reminders = 0;
                *last_reminder = Some(Instant::now());
            } else {
                *empty_reminders += 1;
            }
        }
    }

    Ok(())
}

fn notification(title: &str, body: &str) -> color_eyre::Result {
    Notification::new()
        .appname("skye")
        .summary(title)
        .body(body)
        .sound_name("dialog-information")
        .hint(notify_rust::Hint::SuppressSound(false))
        .urgency(Urgency::Normal)
        .finalize()
        .show()?;
    Ok(())
}
