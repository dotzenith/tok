use anyhow::{anyhow, Context, Result};
use jiff::{civil::DateTime, tz::TimeZone, Timestamp, ToSpan, Zoned};
use reqwest::blocking::Client;
use reqwest::header::{self, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use bincode::{deserialize_from, serialize_into};
use platform_dirs::AppDirs;
use std::fs::{create_dir, OpenOptions};
use std::io::{BufReader, BufWriter};

use open;
use std::env;
use std::sync::mpsc;
use std::thread;
use tiny_http::{Response, Server};

const BASE_AUTH_URL: &'static str = "https://ticktick.com/oauth";
const BASE_API_URL: &'static str = "https://api.ticktick.com";
const SCOPE: &'static str = "tasks:write tasks:read";

/*
We don't need/want all the info given by the API.
Knowing when the token expires is also more useful
than knowing how long until it expires
*/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccessToken {
    pub access_token: String,
    #[serde(with = "jiff::fmt::serde::timestamp::second::required")]
    pub expires_on: Timestamp,
}

#[derive(Debug, Clone)]
pub struct TickTickClient {
    http_client: Client,
}

/*
Everything related to auth and managing the token
*/
impl TickTickClient {
    pub fn new() -> Result<Self> {
        let access_token = match Self::read_access_token() {
            Ok(token) => token,
            Err(_) => Self::get_access_token_from_user()?,
        };

        let mut headers = HeaderMap::new();
        let mut auth_header = HeaderValue::from_str(format!("Bearer {}", access_token.access_token).as_str())
            .context("Unable to build auth header")?;
        auth_header.set_sensitive(true);
        headers.insert(header::AUTHORIZATION, auth_header);
        let http_client = Client::builder().default_headers(headers).build()?;

        Ok(Self { http_client })
    }

    fn get_access_token_from_user() -> Result<AccessToken> {
        let client_id = env::var("TICKTICK_CLIENT_ID").context("Did not find ticktick client id")?;
        let client_secret = env::var("TICKTICK_CLIENT_SECRET").context("Did not find ticktick client secret")?;
        let redirect_url = env::var("TICKTICK_REDIRECT_URL").context("Did not find tictick redirect url")?;
        let auth_url = format!("{BASE_AUTH_URL}/authorize?scope={SCOPE}&client_id={client_id}&state=state&redirect_uri={redirect_url}&response_type=code");

        open::that(&auth_url)?;

        let auth_code = Self::listen_for_redirect(8000)?;
        let access_token = Self::exchange_code_for_token(&client_id, &client_secret, &auth_code, &redirect_url)?;

        let _ = Self::save_access_token(&access_token);

        Ok(access_token)
    }

    pub fn save_access_token(token: &AccessToken) -> Result<()> {
        let app_dirs = AppDirs::new(Some("tok"), true).context("Unable to get cache directory")?;
        if !app_dirs.cache_dir.exists() {
            create_dir(&app_dirs.cache_dir).context("Unable to create cache directory")?;
        }

        let mut file = BufWriter::new(
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(app_dirs.cache_dir.join("tok"))
                .context("Unable to create cache file")?,
        );

        serialize_into(&mut file, token).context("Unable to save token to file")?;
        Ok(())
    }

    pub fn read_access_token() -> Result<AccessToken> {
        let app_dirs = AppDirs::new(Some("tok"), true).context("Unable to get cache directory")?;
        let mut file = BufReader::new(
            OpenOptions::new()
                .read(true)
                .open(app_dirs.cache_dir.join("tok"))
                .context("tok cache does not exist")?,
        );

        let token: AccessToken = deserialize_from(&mut file).context("Unable to read IP from file")?;

        if Timestamp::now() > token.expires_on {
            return Err(anyhow!("Token expired"));
        }
        Ok(token)
    }

    fn listen_for_redirect(port: u16) -> Result<String> {
        let (tx, rx) = mpsc::channel();

        let server = Server::http(format!("127.0.0.1:{}", port))
            .map_err(|e| anyhow::anyhow!("{}", e))
            .context("Failed to start server")?;

        // Clone server for move into thread
        let server = std::sync::Arc::new(server);
        let server_clone = server.clone();

        thread::spawn(move || {
            for request in server_clone.incoming_requests() {
                let url = request.url().to_string();

                let params: HashMap<String, String> =
                    url.split('?')
                        .nth(1)
                        .unwrap_or("")
                        .split('&')
                        .fold(HashMap::new(), |mut dict, param| {
                            let mut parts = param.split('=');
                            match (parts.next(), parts.next()) {
                                (Some(key), Some(value)) => {
                                    dict.insert(key.to_string(), value.to_string());
                                    ()
                                }
                                _ => (),
                            };
                            dict
                        });

                let response = Response::from_string("This window can be closed now").with_status_code(200);
                let _ = request.respond(response);

                let _ = tx.send(params);
                break;
            }
        });

        let params = rx.recv().context("Failed to receive data from the redirect")?;
        Ok(params
            .get("code")
            .ok_or(anyhow!("No code in the redirect"))?
            .to_string())
    }

    fn exchange_code_for_token(
        client_id: &str,
        client_secret: &str,
        code: &str,
        redirect_uri: &str,
    ) -> Result<AccessToken> {
        let http_client = Client::new();
        let mut form = HashMap::new();

        form.insert("client_id", client_id);
        form.insert("client_secret", client_secret);
        form.insert("code", code);
        form.insert("grant_type", "authorization_code");
        form.insert("redirect_uri", redirect_uri);

        let response = http_client.post(format!("{BASE_AUTH_URL}/token")).form(&form).send()?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Request failed with status {}: {}",
                response.status(),
                response.text()?
            ));
        }

        let result: Value = response.json()?;
        let access_token = result["access_token"]
            .as_str()
            .ok_or(anyhow!("Access token not found in api response"))?
            .to_string();

        let expires_in = result["expires_in"]
            .as_i64()
            .ok_or(anyhow!("Token lifetime not found in api response"))?;

        let expires_on = Timestamp::now().checked_add(expires_in.seconds())?;

        Ok(AccessToken {
            access_token,
            expires_on,
        })
    }
}

pub fn deserialize_dt<'de, D>(deserializer: D) -> Result<Option<Zoned>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer)?
        .map(|strtime| {
            let dt = DateTime::strptime("%Y-%m-%dT%H:%M:%S%.3f%z", strtime).map_err(serde::de::Error::custom)?;
            Ok(dt.to_zoned(TimeZone::system()).map_err(serde::de::Error::custom)?)
        })
        .transpose()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub color: String,
    #[serde(rename = "sortOrder")]
    pub sort_order: i64,
    pub closed: Option<bool>,
    #[serde(rename = "groupId")]
    pub group_id: Option<String>,
    #[serde(rename = "viewMode")]
    pub view_mode: Option<String>,
    pub permission: Option<String>,
    pub kind: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChecklistItem {
    pub id: String,
    pub title: String,
    pub status: i32,
    #[serde(rename = "completedTime", deserialize_with = "deserialize_dt", default)]
    pub completed_time: Option<Zoned>,
    #[serde(rename = "isAllDay")]
    pub is_all_day: bool,
    #[serde(rename = "sortOrder")]
    pub sort_order: i64,
    #[serde(rename = "startDate", deserialize_with = "deserialize_dt", default)]
    pub start_date: Option<Zoned>,
    #[serde(rename = "timeZone")]
    pub time_zone: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Task {
    pub id: String,
    #[serde(rename = "projectId")]
    pub project_id: String,
    pub title: String,
    #[serde(rename = "isAllDay")]
    pub is_all_day: bool,
    #[serde(rename = "completedTime", deserialize_with = "deserialize_dt", default)]
    pub completed_time: Option<Zoned>,
    pub content: String,
    pub desc: String,
    #[serde(rename = "dueDate", deserialize_with = "deserialize_dt", default)]
    pub due_date: Option<Zoned>,
    pub items: Option<Vec<ChecklistItem>>,
    pub priority: i32,
    pub reminders: Option<Vec<String>>,
    #[serde(rename = "repeatFlag")]
    pub repeat_flag: Option<String>,
    #[serde(rename = "sortOrder")]
    pub sort_order: i64,
    #[serde(rename = "startDate", deserialize_with = "deserialize_dt", default)]
    pub start_date: Option<Zoned>,
    pub status: u32,
    #[serde(rename = "timeZone")]
    pub time_zone: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Column {
    pub id: String,
    #[serde(rename = "projectId")]
    pub project_id: String,
    pub name: String,
    #[serde(rename = "sortOrder")]
    pub sort_order: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectData {
    pub project: Project,
    pub tasks: Vec<Task>,
    pub columns: Vec<Column>,
}

// API requests
impl TickTickClient {
    pub fn get_projects(&self) -> Result<Vec<Project>> {
        let result: Vec<Project> = self
            .http_client
            .get(format!("{BASE_API_URL}/open/v1/project"))
            .send()?
            .json()?;
        Ok(result)
    }

    pub fn get_projects_with_data(&self) -> Result<Vec<ProjectData>> {
        let projects = self.get_projects()?;
        let mut project_data: Vec<ProjectData> = vec![];

        for project in projects.iter() {
            let result: ProjectData = self
                .http_client
                .get(format!("{BASE_API_URL}/open/v1/project/{}/data", project.id))
                .send()?
                .json()?;
            project_data.push(result);
        }
        Ok(project_data)
    }
}
