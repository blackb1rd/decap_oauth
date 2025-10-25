//! External OAuth provider for Decap CMS. The following environment variables must be set for it to
//! work:
//!
//! ```shell
//! OAUTH_CLIENT_ID=(insert_the_client_id)
//! OAUTH_SECRET=(insert_the_secret)
//! OAUTH_ORIGINS=www.example.com,oauth.mysite.com
//! ```
//!
//! When using GitHub Enterprise, please set `OAUTH_HOSTNAME` to the proper value.

use oauth2::{
    AuthUrl, ClientId, ClientSecret, HttpRequest, HttpResponse, RedirectUrl, TokenUrl,
    basic::BasicClient,
};
use std::env;
use std::future::Future;
use std::pin::Pin;

mod handlers;
pub mod router;

type HttpClient = Box<
    dyn Fn(
            HttpRequest,
        ) -> Pin<
            Box<
                dyn Future<Output = Result<HttpResponse, oauth2::reqwest::Error<reqwest::Error>>>
                    + Send,
            >,
        > + Send
        + Sync,
>;

#[derive(Clone)]
pub struct AppState {
    http_client: std::sync::Arc<HttpClient>,
}

impl AppState {
    pub fn new<F>(http_client: F) -> Self
    where
        F: Fn(
                HttpRequest,
            ) -> Pin<
                Box<
                    dyn Future<
                            Output = Result<HttpResponse, oauth2::reqwest::Error<reqwest::Error>>,
                        > + Send,
                >,
            > + Send
            + Sync
            + 'static,
    {
        Self {
            http_client: std::sync::Arc::new(Box::new(http_client)),
        }
    }
}

const OAUTH_HOSTNAME: &str = "https://github.com";
const OAUTH_TOKEN_PATH: &str = "/login/oauth/access_token";
const OAUTH_AUTHORIZE_PATH: &str = "/login/oauth/authorize";
const OAUTH_SCOPES: &str = "repo";
const OAUTH_PROVIDER: &str = "github";

fn get_var(var: &str) -> String {
    env::var(var).unwrap_or_else(|_| panic!("{} environment variable should be defined", var))
}

fn create_client(redirect_url: String) -> BasicClient {
    let client_id = get_var("OAUTH_CLIENT_ID");
    let secret = get_var("OAUTH_SECRET");
    let hostname = env::var("OAUTH_HOSTNAME").unwrap_or_else(|_| OAUTH_HOSTNAME.to_owned());

    BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(secret)),
        AuthUrl::new(format!("{}{}", hostname, OAUTH_AUTHORIZE_PATH))
            .expect("Auth URL should be a valid URL"),
        Some(
            TokenUrl::new(format!("{}{}", hostname, OAUTH_TOKEN_PATH))
                .expect("Token URL should be a valid URL"),
        ),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url).expect("Invalid redirect URL"))
}

#[cfg(test)]
mod tests {
    use super::router::oauth_router;
    use crate::AppState;
    use axum::http::StatusCode;
    use http_body_util::BodyExt;
    use oauth2::{HttpRequest, HttpResponse};
    use std::env;
    use std::future;
    use std::pin::Pin;
    use tower::util::ServiceExt;

    fn mock_http_client(
        _req: HttpRequest,
    ) -> Pin<
        Box<
            dyn future::Future<
                    Output = Result<HttpResponse, oauth2::reqwest::Error<reqwest::Error>>,
                > + Send,
        >,
    > {
        let content = serde_json::to_string(&serde_json::json!({
            "access_token": "test_token",
            "token_type": "bearer"
        }))
        .unwrap();
        let response = HttpResponse {
            status_code: oauth2::http::StatusCode::OK,
            headers: oauth2::http::HeaderMap::new(),
            body: content.into_bytes(),
        };
        Box::pin(future::ready(Ok(response)))
    }

    fn mock_http_client_error(
        _req: HttpRequest,
    ) -> Pin<
        Box<
            dyn future::Future<
                    Output = Result<HttpResponse, oauth2::reqwest::Error<reqwest::Error>>,
                > + Send,
        >,
    > {
        Box::pin(future::ready(Err(oauth2::reqwest::Error::Reqwest(
            reqwest::Client::new().get("").build().err().unwrap(),
        ))))
    }

    #[tokio::test]
    async fn test_auth() {
        // env::set_var is considered unsafe in this build environment; wrap in unsafe block.
        unsafe {
            env::set_var("OAUTH_CLIENT_ID", "1234");
            env::set_var("OAUTH_SECRET", "5678");
        }

        let state = AppState::new(|_| panic!("http_client should not be called in this test"));
        let app = oauth_router(state);

        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .method("GET")
                    .uri("/auth")
                    .header("Host", "test.com")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        let location = response
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(location.starts_with("https://github.com/login/oauth/authorize?"));
        assert!(location.contains("response_type=code"));
        assert!(location.contains("client_id=1234"));
        assert!(location.contains("redirect_uri=https%3A%2F%2Ftest.com%2Fcallback"));
        assert!(location.contains("scope=repo"));
        assert!(location.contains("state="));
    }

    #[tokio::test]
    async fn test_auth_with_scope() {
        // env::set_var is considered unsafe in this build environment; wrap in unsafe block.
        unsafe {
            env::set_var("OAUTH_CLIENT_ID", "1234");
            env::set_var("OAUTH_SECRET", "5678");
        }

        let state = AppState::new(|_| panic!("http_client should not be called in this test"));
        let app = oauth_router(state);

        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .method("GET")
                    .uri("/auth?scope=custom_scope")
                    .header("Host", "test.com")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        let location = response
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(location.starts_with("https://github.com/login/oauth/authorize?"));
        assert!(location.contains("response_type=code"));
        assert!(location.contains("client_id=1234"));
        assert!(location.contains("redirect_uri=https%3A%2F%2Ftest.com%2Fcallback"));
        assert!(location.contains("scope=custom_scope"));
        assert!(location.contains("state="));
    }

    #[tokio::test]
    async fn test_callback_no_code() {
        // env::set_var is considered unsafe in this build environment; wrap in unsafe block.
        unsafe {
            env::set_var("OAUTH_CLIENT_ID", "1234");
            env::set_var("OAUTH_SECRET", "5678");
        }

        let state = AppState::new(|_| panic!("http_client should not be called in this test"));
        let app = oauth_router(state);

        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .method("GET")
                    .uri("/callback")
                    .header("Host", "test.com")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"Code is required");
    }

    #[tokio::test]
    async fn test_callback_with_code() {
        // env::set_var is considered unsafe in this build environment; wrap in unsafe block.
        unsafe {
            env::set_var("OAUTH_CLIENT_ID", "1234");
            env::set_var("OAUTH_SECRET", "5678");
            env::set_var("OAUTH_ORIGINS", "test.com");
        }

        let state = AppState::new(|req| Box::pin(mock_http_client(req)));
        let app = oauth_router(state);

        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .method("GET")
                    .uri("/callback?code=test_code")
                    .header("Host", "test.com")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains(
            "authorization:github:success:{\"token\":\"test_token\",\"provider\":\"github\"}"
        ));
    }

    #[tokio::test]
    async fn test_callback_with_error() {
        // env::set_var is considered unsafe in this build environment; wrap in unsafe block.
        unsafe {
            env::set_var("OAUTH_CLIENT_ID", "1234");
            env::set_var("OAUTH_SECRET", "5678");
        }

        let state = AppState::new(|req| Box::pin(mock_http_client_error(req)));
        let app = oauth_router(state);

        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .method("GET")
                    .uri("/callback?code=test_code")
                    .header("Host", "test.com")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
