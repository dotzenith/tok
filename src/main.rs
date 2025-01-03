mod client;
mod data;

use kolorz::HexKolorize;

fn main() {
    let tick = client::TickTickClient::new().unwrap();
    let projects = tick.get_projects_with_data().unwrap();

    for project in projects.iter() {
        if !project.tasks.is_empty() {
            println!("{:<16}{:<20}Task", "Project", "Due Date");
        }
    }
}
