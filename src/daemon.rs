use std::{
    fs, io,
    path::PathBuf,
    pin::Pin,
    sync::LazyLock,
    time::{Duration, Instant}
};

use color_eyre::eyre::Context;
use futures::{FutureExt as _, future::pending};
use jiff::{Zoned, civil::Time};
use log::{error, info};
use notify_rust::{Notification, Urgency};
use smol::{
    future::FutureExt as _,
    io::AsyncReadExt,
    net::unix::{SocketAddr, UnixListener, UnixStream}
};

use crate::{
    item::Item,
    negative_habit::{Milestone, span_text},
    positive_habit::{self, PositiveHabit},
    social_habit::{self, SocialHabit},
    store::Store,
    task::{self, Task}
};

enum Command {
    AcceptStream(io::Result<(UnixStream, SocketAddr)>),
    ReloadStore(color_eyre::Result<Store>),
    NegativeHabitRecord { name: String },
    Milestone { milestone: Milestone, name: String },
    NextDay,
    Reminder
}

pub static SOCKET_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    dirs::state_dir()
        .or(dirs::data_dir())
        .unwrap()
        .join("daemon.socket")
});

pub struct Daemon {
    store: Store,
    last_reminder: Option<Instant>,
    remider_duration: Duration,
    empty_reminders: u64,
    listener: UnixListener,
    store_future: Pin<Box<dyn Future<Output = color_eyre::Result<Store>>>>,
    now: Zoned
}

impl Daemon {
    pub fn run(store: Store, remider_duration: Duration) -> color_eyre::Result {
        if SOCKET_PATH.exists() {
            error!("Socket already exists, removing it");
            fs::remove_file(&*SOCKET_PATH)?;
        } else {
            fs::create_dir_all(SOCKET_PATH.parent().unwrap())?;
        }

        let mut daemon = Daemon {
            store,
            last_reminder: None,
            remider_duration,
            empty_reminders: 0,
            listener: UnixListener::bind(&*SOCKET_PATH).wrap_err("Cannot create socket")?,
            store_future: Box::pin(pending()),
            now: Zoned::now()
        };
        loop {
            let command = smol::block_on(daemon.command());
            if let Err(error) = daemon.on_command(command) {
                error!("Daemon error: {error}");
                notification("Daemon error", &error.to_string())?;
            }
        }
    }

    async fn command(&mut self) -> Command {
        self.now = Zoned::now();
        let accept = self.listener.accept().map(Command::AcceptStream);
        let reload = self.store_future.as_mut().map(Command::ReloadStore);
        let record = self
            .store
            .negative_habits
            .iter()
            .filter_map(|(name, habit)| {
                let date_time = self.now.datetime() - habit.span(&self.now) + habit.record()?;
                (!habit.record_notified).then_some((name, date_time))
            })
            .min_by_key(|(_, finish)| *finish);
        let record = async {
            match record {
                Some((name, finish)) => {
                    smol::Timer::after(
                        self.now
                            .duration_until(&finish.to_zoned(self.now.time_zone().clone()).unwrap())
                            .try_into()
                            .unwrap_or_default()
                    )
                    .map(|_| Command::NegativeHabitRecord { name: name.clone() })
                    .await
                }
                None => pending().await
            }
        };
        let reminder =
            smol::Timer::at(self.last_reminder.map_or(Instant::now(), |last_reminder| {
                last_reminder + self.remider_duration + Duration::from_secs(self.empty_reminders)
            }))
            .map(|_| Command::Reminder);
        let tommorow = self
            .now
            .date()
            .tomorrow()
            .unwrap()
            .to_datetime(Time::midnight());
        let duration_till_next_day = self
            .now
            .datetime()
            .duration_until(tommorow)
            .try_into()
            .unwrap();
        let next_day = smol::Timer::after(duration_till_next_day).map(|_| Command::NextDay);
        let milestone = async {
            match self
                .store
                .negative_habits
                .iter()
                .map(|(name, habit)| (name, habit.next_milestone(&self.now)))
                .min_by_key(|(_, (_, timestamp))| *timestamp)
            {
                Some((name, (milestone, finish))) => {
                    let duration: Duration = self
                        .now
                        .timestamp()
                        .duration_until(finish)
                        .try_into()
                        .unwrap_or_default();
                    smol::Timer::after(duration)
                        .map(|_| Command::Milestone {
                            milestone,
                            name: name.clone()
                        })
                        .await
                }
                None => pending().await
            }
        };

        accept
            .or(reload)
            .or(reminder)
            .or(milestone)
            .or(next_day)
            .or(record)
            .await
    }

    fn on_command(&mut self, command: Command) -> Result<(), color_eyre::eyre::Error> {
        match command {
            Command::ReloadStore(store) => {
                info!("Reloading");
                self.store = store?;
            }
            Command::Milestone { milestone, name } => {
                let notification_body = format!(
                    "You reached a <b>{milestone}</b> milestone without the bad habit of <b>{name}</b>. Congrats!",
                );
                notification("Milestone", &notification_body)?;

                self.store.negative_habits[&name].last_milestone = Some(milestone);
                self.store.save()?;
            }
            Command::Reminder => {
                let body = &mut String::new();

                let filter = positive_habit::Filter::NotDone;
                let habits = self.section::<PositiveHabit>("<b>Habits: </b>", filter, body);

                let filter = social_habit::Filter::Pending;
                let social = self.section::<SocialHabit>("\n<b>Social: </b>", filter, body);

                let filter = task::Filter::Queued;
                let tasks = self.section::<Task>("\n<b>Tasks: </b>", filter, body);

                if habits || social || tasks {
                    notification("Reminder", body)?;
                    self.empty_reminders = 0;
                    self.last_reminder = Some(Instant::now());
                } else {
                    self.empty_reminders += 1;
                }
            }
            Command::AcceptStream(stream) => {
                let (mut stream, _) = stream?;
                self.store_future = Box::pin(async move {
                    let mut json = String::new();
                    stream.read_to_string(&mut json).await?;
                    let store = serde_json::from_str(&json)?;
                    Ok(store)
                });
            }
            Command::NegativeHabitRecord { name } => {
                notification(
                    "Habit record",
                    &format!(
                        "Record of habit <b>{name}<b> with the duration of <b>{}<b>",
                        span_text(self.store.negative_habits[&name].span(&self.now))
                    )
                )?;
                self.store.negative_habits[&name].record_notified = true;
                self.store.save()?;
            }
            Command::NextDay => {
                info!("Reloading");
                self.store = Store::new()?;
            }
        }
        Ok(())
    }

    fn section<T: Item>(&mut self, title: &str, filter: T::Filter, body: &mut String) -> bool {
        let mut items: Vec<_> = T::get_items_mut(&mut self.store)
            .iter()
            .filter(|(_, item)| item.filter(filter, &self.now))
            .collect();

        if items.is_empty() {
            return false;
        }

        items.sort_by(|(_, a), (_, b)| b.sort(a, &self.now));

        body.push_str(title);
        body.extend(items.into_iter().flat_map(|(item, _)| [", ", item]).skip(1));

        true
    }
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
