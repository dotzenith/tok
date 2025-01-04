mod client;
mod data;
mod helpers;

use clap::{command, Command};
use std::process::exit;

use crate::helpers::{filter, TimeFrame, print_task};

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

    let matches = command!()
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("show")
                .about("Show To-Do items accross projects")
                .subcommand(Command::new("today").about("Get To-Do items due today"))
                .subcommand(Command::new("tomorrow").about("Get To-Do items due today"))
                .subcommand(Command::new("week").about("Get To-Do items due today"))
                .subcommand(Command::new("all").about("Get To-Do items due today"))
                .subcommand_required(true),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("show", show_matches)) => match show_matches.subcommand() {
            Some(("today", _)) => {
                let tagged_tasks = filter(&projects, TimeFrame::Today);
                for task in tagged_tasks.iter() {
                    print_task(task);
                }
            }
            Some(("tomorrow", _)) => {
                let tagged_tasks = filter(&projects, TimeFrame::Tomorrow);
                for task in tagged_tasks.iter() {
                    print_task(task);
                }
            }
            Some(("week", _)) => {
                let tagged_tasks = filter(&projects, TimeFrame::Week);
                for task in tagged_tasks.iter() {
                    print_task(task);
                }
            }
            Some(("all", _)) => {
                let tagged_tasks = filter(&projects, TimeFrame::All);
                for task in tagged_tasks.iter() {
                    print_task(task);
                }
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}
