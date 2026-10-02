use owo_colors::{OwoColorize, Style};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Task {
    pub queued: bool
}

impl Task {
    pub fn display(&self, name: &str) {
        let style = if self.queued {
            Style::new().bold().yellow().italic()
        } else {
            Style::new().italic()
        };

        println!("{}", name.style(style));
    }
}
