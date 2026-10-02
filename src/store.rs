use std::{
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
    sync::LazyLock
};

use indexmap::IndexMap;
use jiff::{Zoned, civil::Date};
use serde::{Deserialize, Serialize};

use crate::{
    negative_habit::NegativeHabit, positive_habit::PositiveHabit, social_habit::SocialHabit,
    task::Task
};

#[derive(Serialize, Deserialize)]
pub struct Store {
    pub positive_habits: IndexMap<String, PositiveHabit>,
    pub negative_habits: IndexMap<String, NegativeHabit>,
    pub social_habits: IndexMap<String, SocialHabit>,
    pub tasks: IndexMap<String, Task>,
    last_date: Date
}

pub static PATH: LazyLock<PathBuf> =
    LazyLock::new(|| dirs::data_dir().unwrap().join("skye/data.json"));

impl Store {
    pub fn new() -> color_eyre::Result<Self> {
        Ok(if Path::new(&*PATH).exists() {
            Self::load()?
        } else {
            Self {
                positive_habits: IndexMap::default(),
                negative_habits: IndexMap::default(),
                social_habits: IndexMap::default(),
                tasks: IndexMap::default(),
                last_date: Zoned::now().date()
            }
        })
    }

    pub fn save(&self) -> color_eyre::Result {
        fs::write(&*PATH, serde_json::to_string(&self).unwrap())?;

        Ok(())
    }

    fn load() -> color_eyre::Result<Self> {
        let mut this: Self = serde_json::from_reader(BufReader::new(File::open(&*PATH)?))?;
        let handle_positive_habits = match this
            .last_date
            .duration_until(Zoned::now().date())
            .as_hours()
        {
            24 => PositiveHabit::on_next_day,
            25.. => PositiveHabit::reset,
            _ => |_: &mut _| ()
        };

        this.positive_habits
            .values_mut()
            .for_each(handle_positive_habits);
        this.last_date = Zoned::now().date();
        Ok(this)
    }
}
