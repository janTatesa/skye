use std::{cmp::Ordering, fmt::Display};

use indexmap::IndexMap;
use jiff::{Span, SpanRelativeTo, Timestamp, Unit, Zoned, ZonedDifference};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

use crate::{
    item::{Item, NoFilter},
    store::Store,
    utils::{self}
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NegativeHabit {
    last_done: Timestamp,
    pub last_milestone: Option<Milestone>,
    record: Option<jiff::Span>
}

impl Default for NegativeHabit {
    fn default() -> Self {
        Self {
            last_done: Timestamp::now(),
            last_milestone: Option::default(),
            record: Option::default()
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Milestone {
    unit: MilestoneUnit,
    amount: u16
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum MilestoneUnit {
    TensOfMinutes,
    Hours,
    Days,
    Weeks,
    Months
}

impl Milestone {
    pub fn next(mut self) -> Self {
        let (max, next) = match self.unit {
            MilestoneUnit::TensOfMinutes => (5, MilestoneUnit::Hours),
            MilestoneUnit::Hours => (23, MilestoneUnit::Days),
            MilestoneUnit::Days => (6, MilestoneUnit::Weeks),
            MilestoneUnit::Weeks => (3, MilestoneUnit::Months),
            MilestoneUnit::Months => {
                return Self {
                    amount: self.amount + 1,
                    unit: MilestoneUnit::Months
                };
            }
        };

        if self.amount == max {
            self.amount = 1;
            self.unit = next;
        } else {
            self.amount += 1;
        }

        self
    }

    pub fn span(self) -> Span {
        match self.unit {
            MilestoneUnit::TensOfMinutes => Span::new().minutes(self.amount * 10),
            MilestoneUnit::Hours => Span::new().hours(self.amount),
            MilestoneUnit::Days => Span::new().days(self.amount),
            MilestoneUnit::Weeks => Span::new().weeks(self.amount),
            MilestoneUnit::Months => Span::new().months(self.amount)
        }
    }
}

impl Display for Milestone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let amount = if let MilestoneUnit::TensOfMinutes = self.unit {
            self.amount * 10
        } else {
            self.amount
        };
        let unit = match self.unit {
            MilestoneUnit::TensOfMinutes => "minutes",
            MilestoneUnit::Hours => "hours",
            MilestoneUnit::Days => "days",
            MilestoneUnit::Weeks => "weeks",
            MilestoneUnit::Months => "months"
        };
        write!(f, "{amount} {unit}")
    }
}

impl NegativeHabit {
    pub fn r#break(&mut self, now: &Zoned) {
        let span = self.span(now);
        let record_msg = if self
            .record()
            .is_some_and(|record| span.compare(record).unwrap() == Ordering::Greater)
        {
            "That is a new record!"
        } else {
            ""
        };
        let record_msg = record_msg.green();
        let record_msg = record_msg.bold();
        println!(
            "Crap. You lasted {} without it. {record_msg}",
            span_text(span)
        );
        let span = now
            .datetime()
            .since(self.last_done.to_zoned(now.time_zone().clone()).datetime())
            .unwrap();
        self.record = match self.record {
            Some(record)
                if span
                    .compare((record, SpanRelativeTo::days_are_24_hours()))
                    .unwrap()
                    == Ordering::Greater =>
            {
                Some(span)
            }
            None => Some(span),
            _ => self.record
        };
        self.last_done = now.timestamp();
        self.last_milestone = None;
    }

    pub fn next_milestone(&self, now: &Zoned) -> (Milestone, Timestamp) {
        let last_done = self.last_done.to_zoned(now.time_zone().clone());

        let default = Milestone {
            amount: 1,
            unit: MilestoneUnit::TensOfMinutes
        };
        let mut milestone = self.last_milestone.map_or(default, Milestone::next);

        // If we havent ran skye for a while multiple milestones may have been crossed
        while let next_milestone = milestone.next()
            && &last_done + next_milestone.span() < *now
        {
            milestone = next_milestone;
        }

        (milestone, (last_done + milestone.span()).into())
    }

    pub fn span(&self, now: &Zoned) -> Span {
        let diff = ZonedDifference::new(now)
            .smallest(Unit::Second)
            .largest(Unit::Day);
        self.last_done
            .to_zoned(now.time_zone().clone())
            .until(diff)
            .unwrap()
    }

    pub fn record(&self) -> Option<Span> {
        self.record
    }
}

fn span_text(span: Span) -> String {
    let color = utils::days_to_color(span.get_days() as u32);
    let time = [
        (span.get_years(), "y"),
        (span.get_months() as i16, "m"),
        (span.get_days() as i16, "d"),
        (span.get_hours() as i16, "h"),
        (span.get_minutes() as i16, "m"),
        (span.get_seconds() as i16, "s")
    ]
    .iter()
    .skip_while(|(u, _)| *u == 0)
    .map(|(u, i)| format!("{u}{i}"))
    .collect::<Vec<_>>()
    .join(" ");
    time.color(color).to_string()
}

impl Item for NegativeHabit {
    type Filter = NoFilter;

    fn get_items_mut(store: &mut Store) -> &mut IndexMap<String, Self> {
        &mut store.negative_habits
    }

    fn show(&self, name: &str, now: &Zoned) {
        let spacer = " • ".bright_black();
        let span = self.span(now);
        let record = self
            .record
            .map(|record| {
                if span.compare(record).unwrap() == Ordering::Less {
                    format!("{spacer}{}", "That's a new record!".green().bold())
                } else {
                    format!("{spacer}Record {}", span_text(record))
                }
            })
            .unwrap_or_default();
        println!(
            "{}{spacer}Time without it: {}{record}",
            name.italic(),
            span_text(span)
        );
    }
}
