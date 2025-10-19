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

use axum::{
    extract::Query,
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    routing, Router,
};
use oauth2::{
    basic::BasicClient, reqwest::http_client, AccessToken, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use std::collections::HashMap;
use std::env;

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

/// The auth route.
pub async fn auth(Query(params): Query<HashMap<String, String>>, headers: HeaderMap) -> Response {
    let scope = match params.get("scope") {
        Some(scope) => scope.to_owned(),
        None => OAUTH_SCOPES.to_string(),
    };

    let host = match headers.get("host") {
        Some(host) => host.to_str().unwrap(),
        None => return (StatusCode::BAD_REQUEST, "No host header".to_string()).into_response(),
    };

    let redirect_url = format!("https://{}/callback", host);

    let client = create_client(redirect_url);

    let (auth_url, _csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(scope))
        .url();

    Redirect::to(&auth_url.to_string()).into_response()
}

fn login_response(status: &str, token: &AccessToken) -> Html<String> {
    let origins = get_var("OAUTH_ORIGINS");

    Html(format!(
        r#"
    <script>
      const receiveMessage = (e) => {{
        let matches = false;

        for(const origin of '{}'.split(',')) {{
          if (e.origin.match(origin)) {{
              matches = true;
              break;
          }}
        }}

        if (!matches) {{
          return;
        }}

        window.opener.postMessage(
          'authorization:{}:{}:{{"token":"{}","provider":"{}"}}',
          e.origin
        );

        window.removeEventListener('message', receiveMessage, false);
      }}
      window.addEventListener('message', receiveMessage, false);

      window.opener.postMessage('authorizing:{}', '*');
    </script>
    "#,
        origins,
        OAUTH_PROVIDER,
        status,
        token.secret(),
        OAUTH_PROVIDER,
        OAUTH_PROVIDER,
    ))
}

/// The callback route.
pub async fn callback(
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Response {
    let code = match params.get("code") {
        Some(code) => AuthorizationCode::new(code.to_string()),
        None => return (StatusCode::BAD_REQUEST, "Code is required".to_string()).into_response(),
    };

    let host = match headers.get("host") {
        Some(host) => host.to_str().unwrap(),
        None => return (StatusCode::BAD_REQUEST, "No host header".to_string()).into_response(),
    };

    let redirect_url = format!("https://{}/callback", host);

    let client = create_client(redirect_url);

    match client.exchange_code(code).request(http_client) {
        Ok(token) => (
            StatusCode::OK,
            login_response("success", token.access_token()),
        )
            .into_response(),
        Err(e) => {
            eprintln!("{:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Return a full Axum router with both routes used by OAuth.
pub fn oauth_router() -> Router {
    Router::new()
        .route("/auth", routing::get(auth))
        .route("/callback", routing::get(callback))
}
