use anyhow::{anyhow, Context, Result};
use jiff::{Timestamp, ToSpan};
use reqwest::blocking::Client;
use reqwest::header::{self, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::data::{Project, ProjectData, Task};
use crate::helpers::generate_state_token;

use bincode::{deserialize_from, serialize_into};
use platform_dirs::AppDirs;
use std::fs::{create_dir, OpenOptions};
use std::io::{BufReader, BufWriter};

use std::env;
use std::sync::mpsc;
use std::thread;
use tiny_http::{Response, Server};

const BASE_AUTH_URL: &str = "https://ticktick.com/oauth";
const BASE_API_URL: &str = "https://api.ticktick.com";
const SCOPE: &str = "tasks:write tasks:read";

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

#[derive(Debug, Clone)]
pub struct AuthRedirect {
    code: String,
    state: String,
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
        /*
        I really don't think we need to bother too much with the state token.
        The server is quite literally meant for a oneshot and any user
        is going to be rolling their own credentials, this is not going to be a
        long running service
        */

        let state = generate_state_token();
        let redirect_url = env::var("TICKTICK_REDIRECT_URL").context("Did not find tictick redirect url")?;
        let address: &str = redirect_url
            .split("//")
            .nth(1)
            .context("Bad redirect_url format")?
            .trim_end_matches("/");
        let client_id = env::var("TICKTICK_CLIENT_ID").context("Did not find ticktick client id")?;
        let client_secret = env::var("TICKTICK_CLIENT_SECRET").context("Did not find ticktick client secret")?;

        let auth_url = format!("{BASE_AUTH_URL}/authorize?scope={SCOPE}&client_id={client_id}&state={state}&redirect_uri={redirect_url}&response_type=code");

        open::that(&auth_url)?;

        let auth_redirect = Self::listen_for_redirect(&address)?;
        let access_token =
            Self::exchange_code_for_token(&client_id, &client_secret, &auth_redirect, &state, &redirect_url)?;

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

    fn listen_for_redirect(address: &str) -> Result<AuthRedirect> {
        let (tx, rx) = mpsc::channel();

        let server = Server::http(address)
            .map_err(|e| anyhow::anyhow!("{}", e))
            .context("Failed to start server")?;

        // Clone server for move into thread
        let server = std::sync::Arc::new(server);
        let server_clone = server.clone();

        thread::spawn(move || {
            if let Some(request) = server_clone.incoming_requests().next() {
                let url = request.url().to_string();

                let params: HashMap<String, String> =
                    url.split('?')
                        .nth(1)
                        .unwrap_or("")
                        .split('&')
                        .fold(HashMap::new(), |mut dict, param| {
                            let mut parts = param.split('=');

                            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                                dict.insert(key.to_string(), value.to_string());
                            };
                            dict
                        });

                let response = Response::from_string("This window can be closed now").with_status_code(200);
                let _ = request.respond(response);

                let _ = tx.send(params);
            }
        });

        let params = rx.recv().context("Failed to receive data from the redirect")?;
        let code = params
            .get("code")
            .ok_or(anyhow!("No code in the redirect"))?
            .to_string();
        let state = params
            .get("state")
            .ok_or(anyhow!("No state in the redirect"))?
            .to_string();

        Ok(AuthRedirect { code, state })
    }

    fn exchange_code_for_token(
        client_id: &str,
        client_secret: &str,
        auth_redirect: &AuthRedirect,
        state: &str,
        redirect_uri: &str,
    ) -> Result<AccessToken> {
        if auth_redirect.state != state {
            return Err(anyhow!("State token does not match"));
        }

        let http_client = Client::new();
        let mut form = HashMap::new();

        form.insert("client_id", client_id);
        form.insert("client_secret", client_secret);
        form.insert("code", &auth_redirect.code);
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

// API requests
impl TickTickClient {
    pub fn get_projects(&self) -> Result<Vec<Project>> {
        self.http_client
            .get(format!("{BASE_API_URL}/open/v1/project"))
            .send()
            .map_err(|e| anyhow!("Failed to send request: {}", e))?
            .error_for_status()
            .map_err(|e| anyhow!("API error: {}", e))?
            .json()
            .map_err(|e| anyhow!("Failed to parse response: {}", e))
    }

    pub fn get_projects_with_data(&self) -> Result<Vec<ProjectData>> {
        let projects = self.get_projects()?;

        projects
            .iter()
            .map(|project| {
                self.http_client
                    .get(format!("{BASE_API_URL}/open/v1/project/{}/data", project.id))
                    .send()
                    .map_err(|e| anyhow!("Failed to fetch project data request for {}: {}", project.id, e))?
                    .error_for_status()
                    .map_err(|e| anyhow!("API error fetching project data for {}: {}", project.id, e))?
                    .json()
                    .map_err(|e| anyhow!("Failed to parse project data for {}: {}", project.id, e))
            })
            .collect()
    }
    pub fn complete_task(&self, task: &Task) -> Result<()> {
        self.http_client
            .post(format!(
                "{BASE_API_URL}/open/v1/project/{}/task/{}/complete",
                task.project_id, task.id
            ))
            .send()
            .map_err(|e| anyhow!("Failed to send complete task request: {}", e))?
            .error_for_status()
            .map_err(|e| anyhow!("Failed to complete task: {}", e))?;

        Ok(())
    }

    pub fn delete_task(&self, task: &Task) -> Result<()> {
        self.http_client
            .delete(format!(
                "{BASE_API_URL}/open/v1/project/{}/task/{}",
                task.project_id, task.id
            ))
            .send()
            .map_err(|e| anyhow!("Failed to send delete task request: {}", e))?
            .error_for_status()
            .map_err(|e| anyhow!("Failed to delete task: {}", e))?;

        Ok(())
    }
}
