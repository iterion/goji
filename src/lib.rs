//! Goji provides an interface for Jira's REST api

#[macro_use]
extern crate log;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate url;


use reqwest::header::CONTENT_TYPE;
use reqwest::{Client, Method, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;

mod builder;
mod errors;
pub mod issues;
mod rep;
mod search;
mod transitions;

pub use crate::builder::*;
pub use crate::errors::*;
pub use crate::issues::*;
pub use crate::rep::*;
pub use crate::search::Search;
pub use crate::transitions::*;
pub mod boards;
pub mod resolution;
pub use crate::boards::*;
pub mod sprints;
pub use crate::sprints::*;

#[derive(Deserialize, Debug)]
pub struct EmptyResponse;

pub type Result<T> = std::result::Result<T, Error>;

/// Types of authentication credentials
#[derive(Clone, Debug)]
pub enum Credentials {
    /// username and password credentials
    Basic(String, String), // todo: OAuth
}

/// Entrypoint into client interface
/// https://docs.atlassian.com/jira/REST/latest/
#[derive(Clone, Debug)]
pub struct Jira {
    host: String,
    credentials: Credentials,
    client: Client,
}

impl Jira {
    /// creates a new instance of a jira client
    pub fn new<H>(host: H, credentials: Credentials) -> Result<Jira>
    where
        H: Into<String>,
    {
        Ok(Jira {
            host: host.into(),
            client: Client::new(),
            credentials,
        })
    }

    /// creates a new instance of a jira client using a specified reqwest client
    pub fn from_client<H>(host: H, credentials: Credentials, client: Client) -> Result<Jira>
    where
        H: Into<String>,
    {
        Ok(Jira {
            host: host.into(),
            credentials,
            client,
        })
    }

    /// return transitions interface
    pub fn transitions<K>(&self, key: K) -> Transitions
    where
        K: Into<String>,
    {
        Transitions::new(self, key)
    }

    /// return search interface
    pub fn search(&self) -> Search {
        Search::new(self)
    }

    // return issues interface
    pub fn issues(&self) -> Issues {
        Issues::new(self)
    }

    // return boards interface
    pub fn boards(&self) -> Boards {
        Boards::new(self)
    }

    // return boards interface
    pub fn sprints(&self) -> Sprints {
        Sprints::new(self)
    }

    async fn post<D, S>(&self, api_name: &str, endpoint: &str, body: S) -> Result<D>
    where
        D: DeserializeOwned,
        S: Serialize,
    {
        let data = serde_json::to_string::<S>(&body)?;
        debug!("Json request: {}", data);
        self.request::<D>(Method::POST, api_name, endpoint, Some(data.into_bytes())).await
    }

    async fn put<D, S>(&self, api_name: &str, endpoint: &str, body: S) -> Result<D>
    where
        D: DeserializeOwned,
        S: Serialize,
    {
        let data = serde_json::to_string::<S>(&body)?;
        debug!("Json request: {}", data);
        self.request::<D>(Method::PUT, api_name, endpoint, Some(data.into_bytes())).await
    }

    async fn get<D>(&self, api_name: &str, endpoint: &str) -> Result<D>
    where
        D: DeserializeOwned,
    {
        self.request::<D>(Method::GET, api_name, endpoint, None).await
    }

    async fn execute(
        &self,
        method: Method,
        api_name: &str,
        endpoint: &str,
        body: Option<Vec<u8>>,
    ) -> Result<reqwest::Response> {
        let url = format!("{}/rest/{}/latest{}", self.host, api_name, endpoint);
        debug!("url -> {:?}", url);

        let req = self.client.request(method, &url);
        let builder = match self.credentials {
            Credentials::Basic(ref user, ref pass) => req
                .basic_auth(user.to_owned(), Some(pass.to_owned()))
                .header(CONTENT_TYPE, "application/json"),
        };

        let resp = match body {
            Some(bod) => builder.body(bod).send().await,
            _ => builder.send().await,
        };

        match resp {
            Ok(ok) => Ok(ok),
            Err(err) => Err(err.into())
        }
    }

    async fn request<D>(
        &self,
        method: Method,
        api_name: &str,
        endpoint: &str,
        body: Option<Vec<u8>>,
    ) -> Result<D>
    where
        D: DeserializeOwned,
    {
        let res = self.execute(method, api_name, endpoint, body).await?;

        match res.status() {
            StatusCode::UNAUTHORIZED => Err(Error::Unauthorized),
            StatusCode::METHOD_NOT_ALLOWED => Err(Error::MethodNotAllowed),
            StatusCode::NOT_FOUND => Err(Error::NotFound),
            client_err if client_err.is_client_error() => Err(Error::Fault {
                code: res.status(),
                errors: res.json::<Errors>().await?,
            }),
            _ => {
                Ok(res.json::<D>().await?)
            }
        }
    }
}
