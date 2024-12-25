use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Duration, Utc};
use reqwest::blocking::Client;
use reqwest::header::{self, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

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
struct AccessToken {
    pub access_token: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub expires_on: DateTime<Utc>,
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
        let client_id = env::var("TICKTICK_CLIENT_ID").context("Did not find ticktick client id")?;
        let client_secret = env::var("TICKTICK_CLIENT_SECRET").context("Did not find ticktick client secret")?;
        let redirect_url = env::var("TICKTICK_REDIRECT_URL").context("Did not find tictick redirect url")?;
        let auth_url = format!("{BASE_AUTH_URL}/authorize?scope={SCOPE}&client_id={client_id}&state=state&redirect_uri={redirect_url}&response_type=code");

        open::that(&auth_url)?;

        let auth_code = Self::listen_for_redirect(8000)?;

        let access_token = Self::exchange_code_for_token(&client_id, &client_secret, &auth_code, &redirect_url)?;

        let mut headers = HeaderMap::new();
        let mut auth_header = HeaderValue::from_str(format!("Bearer {}", access_token.access_token).as_str())
            .context("Unable to build auth header")?;
        auth_header.set_sensitive(true);
        headers.insert(header::AUTHORIZATION, auth_header);
        let http_client = Client::builder().default_headers(headers).build()?;

        Ok(Self { http_client })
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

        let expires_on = Utc::now() + Duration::seconds(expires_in);

        Ok(AccessToken {
            access_token,
            expires_on,
        })
    }
}

// API requests
impl TickTickClient {
    pub fn get_projects(&self) -> Result<Value> {
        let request: Value = self
            .http_client
            .get(format!("{BASE_API_URL}/open/v1/project"))
            .send()?
            .json()?;
        Ok(request)
    }
}
