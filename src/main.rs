mod client;

fn main() {
    let new_client = client::TickTickClient::new().unwrap();
    let projects = new_client.get_projects().unwrap();
    println!("projects: {:?}", projects);
}
