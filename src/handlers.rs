use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
};
use oauth2::{AccessToken, AuthorizationCode, CsrfToken, Scope, TokenResponse};
use std::collections::HashMap;

use crate::{AppState, OAUTH_PROVIDER, OAUTH_SCOPES, create_client, get_var};

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

    Redirect::to(auth_url.as_ref()).into_response()
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
    State(state): State<AppState>,
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

    match client
        .exchange_code(code)
        .request_async(|req| (state.http_client)(req))
        .await
    {
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
