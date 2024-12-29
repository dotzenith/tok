mod client;

fn main() {
    let new_client = client::TickTickClient::new().unwrap();
    let projects = new_client.get_projects_with_data().unwrap();
    for project in projects.iter() {
        println!("{:?}\n", project);
    }
}
