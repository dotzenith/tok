use jiff::{Span, Unit, Zoned};

use crate::data::{ProjectData, Task};
use kolorz::HexKolorize;

pub struct TaggedTask<'a> {
    pub project_name: &'a str,
    pub color: Option<&'a str>,
    pub task: &'a Task,
}

pub enum TimeFrame {
    Today,
    Tomorrow,
    Week,
    All,
}

impl TimeFrame {
    pub fn inside(&self, today: &Zoned, due: &Zoned) -> bool {
        let span: Span = due - today;
        let days = span
            .total(Unit::Day)
            .expect("Could not get total days between now and due date");
        match self {
            Self::Today => days < 1.0,
            Self::Tomorrow => days > 1.0 && days < 2.0,
            Self::Week => days < 7.0,
            Self::All => true,
        }
    }
}

pub fn print_task(num: usize, tagged_task: &TaggedTask) {
    match tagged_task.color {
        Some(col) => {
            println!(
                "{} {:<16} {} [{}]",
                format!("({:02})", num + 1),
                tagged_task
                    .task
                    .due_date
                    .as_ref()
                    .expect("No due date")
                    .strftime("[%m/%d %I:%M %p]")
                    .to_string(),
                tagged_task.task.title,
                tagged_task.project_name.kolorize(col)
            );
        }
        None => {
            println!(
                "{} {:<16} {} [{}]",
                format!("({:02})", num + 1),
                tagged_task
                    .task
                    .due_date
                    .as_ref()
                    .expect("No due date")
                    .strftime("[%m/%d %I:%M %p]")
                    .to_string(),
                tagged_task.task.title,
                tagged_task.project_name
            );
        }
    }
}

pub fn filter(projects: &[ProjectData], frame: TimeFrame) -> Vec<TaggedTask<'_>> {
    let today = Zoned::now();
    let mut result: Vec<TaggedTask> = vec![];
    for project in projects.iter() {
        for task in project.tasks.iter() {
            if let Some(date) = &task.due_date {
                if frame.inside(&today, date) {
                    let tagged_task = TaggedTask {
                        project_name: &project.project.name,
                        color: project.project.color.as_deref(),
                        task,
                    };
                    result.push(tagged_task);
                }
            }
        }
    }
    result
}
