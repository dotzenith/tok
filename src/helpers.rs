use anyhow::{anyhow, Context, Result};
use jiff::{Span, Unit, Zoned};
use rand::Rng;
use std::fmt::Write as FmtWrite;
use std::io::{self, Write};

use crate::data::{ProjectData, Task};
use kolorz::HexKolorize;

pub struct TaggedTask<'a> {
    pub project_name: &'a str,
    pub color: Option<&'a str>,
    pub task: &'a Task,
}

#[derive(Clone, Copy)]
pub enum TimeFrame {
    Today,
    Tomorrow,
    Week,
    All,
}

impl TimeFrame {
    pub fn inside(&self, today: &Zoned, due: &Zoned) -> bool {
        match self {
            Self::Today => {
                // The Today section in TickTick includes past due
                today.day_of_year() >= due.day_of_year() && today.year() == due.year()
            }
            Self::Tomorrow => {
                let tomorrow = today.tomorrow().expect("Tomorrow doesn't exist?");
                tomorrow.day_of_year() == due.day_of_year() && tomorrow.year() == due.year()
            }
            Self::Week => {
                let span: Span = due - today;
                let days = span
                    .total(Unit::Day)
                    .expect("Could not get total days between now and due date");
                days < 7.0
            }
            Self::All => true,
        }
    }
}

pub fn print_task(num: usize, tagged_task: &TaggedTask, now: &Zoned) {
    let time = tagged_task.task.due_date.as_ref().unwrap_or(now);
    match tagged_task.color {
        Some(col) => {
            println!(
                "({:03}) {:<16} {} [{}]",
                num + 1,
                time.strftime("[%m/%d %I:%M %p]").to_string(),
                tagged_task.task.title,
                tagged_task.project_name.kolorize(col)
            );
        }
        None => {
            println!(
                "({:03}) {:<16} {} [{}]",
                num + 1,
                time.strftime("[%m/%d %I:%M %p]").to_string(),
                tagged_task.task.title,
                tagged_task.project_name
            );
        }
    }
}

pub fn filter(projects: &[ProjectData], frame: TimeFrame) -> Vec<TaggedTask<'_>> {
    let today = Zoned::now();

    projects
        .iter()
        .flat_map(|proj| {
            let value = today.clone();
            proj.tasks.iter().filter_map({
                move |task| {
                    let should_include = match &task.due_date {
                        Some(date) => frame.inside(&value, date),
                        None => matches!(frame, TimeFrame::All),
                    };

                    should_include.then(|| TaggedTask {
                        project_name: &proj.project.name,
                        color: proj.project.color.as_deref(),
                        task,
                    })
                }
            })
        })
        .collect()
}

pub fn get_number(max: usize) -> Result<usize> {
    /*
    I really am just asking for off-by-one errors here
    but zero indexing looks funny so I'd rather not do
    that instead
    */
    print!("Please enter a task number: ");
    io::stdout()
        .flush()
        .context("Could not flush stdout while asking for user input")?;
    let mut input = String::new();

    io::stdin().read_line(&mut input).context("Could not get user input")?;

    let num: i64 = input.trim().parse().context("Task number wasn't a number")?;

    if num < 1 || num > max as i64 {
        Err(anyhow!("Invalid task number"))
    } else {
        Ok((num - 1) as usize)
    }
}

pub fn generate_state_token() -> String {
    let mut rng = rand::thread_rng();
    (0..32).fold(String::new(), |mut output, _| {
        let _ = write!(output, "{:02x}", rng.gen::<u8>());
        output
    })
}
