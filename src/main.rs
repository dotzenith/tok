mod client;
mod data;
mod helpers;

use clap::{arg, command, value_parser, Command};
use jiff::Zoned;
use std::process::exit;

use crate::helpers::{filter, get_number, print_task, TimeFrame};

use self::client::TickTickClient;
use self::data::ProjectData;

fn main() {
    let tick = match client::TickTickClient::new() {
        Ok(client) => client,
        Err(err) => {
            eprintln!("{}", err);
            exit(1)
        }
    };

    let projects = match tick.get_projects_with_data() {
        Ok(proj) => proj,
        Err(err) => {
            eprintln!("{}", err);
            exit(1)
        }
    };

    let now = Zoned::now();

    let matches = command!()
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("show")
                .about("Show To-Do items accross projects")
                .subcommand(Command::new("today").about("To-Do items due today"))
                .subcommand(Command::new("tomorrow").about("To-Do items due tomorrow"))
                .subcommand(Command::new("week").about("To-Do items due this week"))
                .subcommand(Command::new("all").about("All To-Do items"))
                .arg(
                    arg!(--project <NAME>)
                        .help("Project name to filter by")
                        .value_parser(value_parser!(String))
                        .global(true)
                        .require_equals(false),
                )
                .subcommand_required(true),
        )
        .subcommand(
            Command::new("complete")
                .about("Complete a given To-Do item accross projects")
                .subcommand(Command::new("today").about("To-Do items due today"))
                .subcommand(Command::new("tomorrow").about("To-Do items due tomorrow"))
                .subcommand(Command::new("week").about("To-Do items due this week"))
                .subcommand(Command::new("all").about("All To-Do items"))
                .arg(
                    arg!(--project <NAME>)
                        .help("Project name to filter by")
                        .value_parser(value_parser!(String))
                        .global(true)
                        .require_equals(false),
                )
                .subcommand_required(true),
        )
        .subcommand(
            Command::new("delete")
                .about("Delete a given To-Do item accross projects")
                .subcommand(Command::new("today").about("To-Do items due today"))
                .subcommand(Command::new("tomorrow").about("To-Do items due tomorrow"))
                .subcommand(Command::new("week").about("To-Do items due this week"))
                .subcommand(Command::new("all").about("All To-Do items"))
                .arg(
                    arg!(--project <NAME>)
                        .help("Project name to filter by")
                        .value_parser(value_parser!(String))
                        .global(true)
                        .require_equals(false),
                )
                .subcommand_required(true),
        )
        .get_matches();

    // Much of this is repetitive but I really don't want to abstract it out to another
    // function. It deals with the commandline directly and I would rather keep the
    // logic right here.
    match matches.subcommand() {
        Some(("show", show_matches)) => match show_matches.subcommand() {
            Some(("today", _)) => {
                let project = show_matches.get_one::<String>("project");
                let tagged_tasks = filter(&projects, TimeFrame::Today, project.map(|x| x.as_str()));
                for (num, task) in tagged_tasks.iter().enumerate() {
                    print_task(num, task, &now);
                }
            }
            Some(("tomorrow", _)) => {
                let project = show_matches.get_one::<String>("project");
                let tagged_tasks = filter(&projects, TimeFrame::Tomorrow, project.map(|x| x.as_str()));
                for (num, task) in tagged_tasks.iter().enumerate() {
                    print_task(num, task, &now);
                }
            }
            Some(("week", _)) => {
                let project = show_matches.get_one::<String>("project");
                let tagged_tasks = filter(&projects, TimeFrame::Week, project.map(|x| x.as_str()));
                for (num, task) in tagged_tasks.iter().enumerate() {
                    print_task(num, task, &now);
                }
            }
            Some(("all", _)) => {
                let project = show_matches.get_one::<String>("project");
                let tagged_tasks = filter(&projects, TimeFrame::All, project.map(|x| x.as_str()));
                for (num, task) in tagged_tasks.iter().enumerate() {
                    print_task(num, task, &now);
                }
            }
            _ => unreachable!(),
        },
        Some(("complete", show_matches)) => match show_matches.subcommand() {
            Some(("today", _)) => {
                let project = show_matches.get_one::<String>("project");
                show_and_finish_task(
                    &projects,
                    project.map(|x| x.as_str()),
                    TimeFrame::Today,
                    TaskAction::Complete,
                    &tick,
                    &now,
                );
            }
            Some(("tomorrow", _)) => {
                let project = show_matches.get_one::<String>("project");
                show_and_finish_task(
                    &projects,
                    project.map(|x| x.as_str()),
                    TimeFrame::Tomorrow,
                    TaskAction::Complete,
                    &tick,
                    &now,
                );
            }
            Some(("week", _)) => {
                let project = show_matches.get_one::<String>("project");
                show_and_finish_task(
                    &projects,
                    project.map(|x| x.as_str()),
                    TimeFrame::Week,
                    TaskAction::Complete,
                    &tick,
                    &now,
                );
            }
            Some(("all", _)) => {
                let project = show_matches.get_one::<String>("project");
                show_and_finish_task(
                    &projects,
                    project.map(|x| x.as_str()),
                    TimeFrame::All,
                    TaskAction::Complete,
                    &tick,
                    &now,
                );
            }
            _ => unreachable!(),
        },
        Some(("delete", show_matches)) => match show_matches.subcommand() {
            Some(("today", _)) => {
                let project = show_matches.get_one::<String>("project");
                show_and_finish_task(
                    &projects,
                    project.map(|x| x.as_str()),
                    TimeFrame::Today,
                    TaskAction::Delete,
                    &tick,
                    &now,
                );
            }
            Some(("tomorrow", _)) => {
                let project = show_matches.get_one::<String>("project");
                show_and_finish_task(
                    &projects,
                    project.map(|x| x.as_str()),
                    TimeFrame::Tomorrow,
                    TaskAction::Delete,
                    &tick,
                    &now,
                );
            }
            Some(("week", _)) => {
                let project = show_matches.get_one::<String>("project");
                show_and_finish_task(
                    &projects,
                    project.map(|x| x.as_str()),
                    TimeFrame::Week,
                    TaskAction::Delete,
                    &tick,
                    &now,
                );
            }
            Some(("all", _)) => {
                let project = show_matches.get_one::<String>("project");
                show_and_finish_task(
                    &projects,
                    project.map(|x| x.as_str()),
                    TimeFrame::All,
                    TaskAction::Delete,
                    &tick,
                    &now,
                );
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}

#[derive(Clone, Copy)]
enum TaskAction {
    Complete,
    Delete,
}

fn show_and_finish_task(
    projects: &[ProjectData],
    project_filter: Option<&str>,
    timeframe: TimeFrame,
    task_action: TaskAction,
    client: &TickTickClient,
    now: &Zoned,
) {
    let tagged_tasks = filter(projects, timeframe, project_filter);
    let num_tasks = tagged_tasks.len();
    for (num, task) in tagged_tasks.iter().enumerate() {
        print_task(num, task, now);
    }

    if num_tasks < 1 {
        return;
    }

    let remove = match get_number(tagged_tasks.len() + 1) {
        Ok(num) => num,
        Err(err) => {
            eprintln!("Error with user input: {}", err);
            exit(1);
        }
    };

    match task_action {
        TaskAction::Complete => match client.complete_task(tagged_tasks[remove].task) {
            Ok(_) => println!("Task completed successfully"),
            Err(err) => eprintln!("Unable to complete task: {}", err),
        },
        TaskAction::Delete => match client.delete_task(tagged_tasks[remove].task) {
            Ok(_) => println!("Task delete successfully"),
            Err(err) => eprintln!("Unable to delete task: {}", err),
        },
    }
}
