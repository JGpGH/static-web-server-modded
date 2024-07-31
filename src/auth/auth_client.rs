use std::time::{Duration, Instant};
use std::sync::RwLock;
use http::header::AUTHORIZATION;
use http::Uri;
use hyper::{Body, Request};
use moka::future::Cache;
use hyper::Client;

pub struct Session {
    pub token: String,
    pub expires_at: Instant,
}

/// AuthClient struct represents an authentication client.
pub struct AuthClient {
    is_member_of_group_cache: Cache<String, bool>,
    session: RwLock<Option<Session>>,
    http_client: Client<hyper::client::HttpConnector>,
    user: String,
    base_path: String,
    password: String,
}

impl AuthClient {
    /// Creates an `AuthClient` from a connection string.
    ///
    /// # Arguments
    ///
    /// * `connection_string` - The connection string used to create the `AuthClient`.
    pub fn from_conn_str(connection_string: &str) -> Result<Self, String> {
        let mut parts = connection_string.splitn(3, '#');

        let user = parts.next().ok_or("Missing username")?;
        let base_path = parts.next().ok_or("Missing URL")?;
        let password = parts.next().ok_or("Missing password")?;

        Ok(Self {
            is_member_of_group_cache: Self::new_member_of_group_cache(),
            session: RwLock::new(None),
            http_client: Client::new(),
            user: user.to_owned(),
            base_path: base_path.to_owned(),
            password: password.to_owned(),
        })
    }

    fn new_member_of_group_cache() -> Cache<String, bool> {
        Cache::builder()
            .max_capacity(1000)
            .time_to_live(Duration::from_secs(120))
            .build()
    }

    /// Checks if a user is a member of a group.
    ///
    /// # Arguments
    ///
    /// * `group` - The name of the group.
    /// * `user` - The name of the user.
    ///
    /// # Returns
    ///
    /// * `Result<bool, String>` - `Ok(true)` if the user is a member of the group, `Ok(false)` otherwise.
    pub async fn is_member_of_group(&mut self, group: &str, user: &str) -> Result<bool, String> {
        if let Some(is_member) = self.is_member_of_group_cache.get(&format!("{}#{}", group, user)).await {
            return Ok(is_member);
        }

        let uri: Uri = format!("{}/is-member/{}/{}", self.base_path, user, group).parse().unwrap();
        let req = Request::builder().uri(uri).header(AUTHORIZATION, self.get_session_token().await?).body(Body::empty()).unwrap();
        let response = self.http_client.request(req).await.map_err(|e| e.to_string())?;

        let body = hyper::body::to_bytes(response.into_body()).await.map_err(|e| e.to_string())?;
        let body_text = String::from_utf8(body.into()).map_err(|e| e.to_string())?;
        return Ok(body_text == "true");
    }

    /// Retrieves the session token.
    ///
    /// # Returns
    ///
    /// * `Result<String, String>` - The session token if successful, or an error message if not.
    async fn get_session_token(&mut self) -> Result<String, String> {
        if let Some(session) = self.session.read().unwrap().as_ref() {
            if session.expires_at > Instant::now() {
                return Ok(session.token.clone());
            }
        }

        let uri: Uri = format!("{}/session?username={}&password={}&service_name={}", self.base_path, self.user, self.password, "authentication").parse().unwrap();

        let response = self.http_client.get(uri).await.map_err(|e| e.to_string())?;

        let body = hyper::body::to_bytes(response.into_body()).await.map_err(|e| e.to_string())?;

        let token = String::from_utf8(body.into()).map_err(|e| e.to_string())?;
        *self.session.write().unwrap() = Some(Session { token: token.clone(), expires_at: Instant::now() + Duration::from_secs(1000) });

        Ok(token)
    }

}
