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

use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl, basic::BasicClient};
use std::env;

mod handlers;
pub mod router;

const OAUTH_HOSTNAME: &str = "https://github.com";
const OAUTH_TOKEN_PATH: &str = "/login/oauth/access_token";
const OAUTH_AUTHORIZE_PATH: &str = "/login/oauth/authorize";
const OAUTH_SCOPES: &str = "repo";
const OAUTH_PROVIDER: &str = "github";

fn get_var(var: &str) -> String {
    env::var(var).expect(format!("{} environment variable should be defined", var).as_str())
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
    use axum::http::StatusCode;
    use http_body_util::BodyExt;
    use std::env;
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn test_auth() {
        env::set_var("OAUTH_CLIENT_ID", "1234");
        env::set_var("OAUTH_SECRET", "5678");

        let app = oauth_router();

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
    async fn test_callback_no_code() {
        env::set_var("OAUTH_CLIENT_ID", "1234");
        env::set_var("OAUTH_SECRET", "5678");

        let app = oauth_router();

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
}
