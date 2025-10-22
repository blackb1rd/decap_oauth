> **Note:** This is a fork of the original [decap-oauth](https://github.com/augustogunsch/decap_oauth) project.

External OAuth provider for Decap CMS. The following environment variables must be set for it to
work:

```shell
OAUTH_CLIENT_ID=(insert_the_client_id)
OAUTH_SECRET=(insert_the_secret)
OAUTH_ORIGINS=www.example.com,oauth.mysite.com
```

When using GitHub Enterprise, please set `OAUTH_HOSTNAME` to the proper value.

Documentation available on [docs.rs](https://docs.rs/decap-cms-oauth/latest/decap_cms_oauth/).

## GitHub OAuth Application Setup

To use this provider, you need to create a new OAuth application on GitHub. Follow these steps:

1.  **Go to GitHub Developer Settings:**
    *   Log in to your GitHub account.
    *   Click on your profile picture in the top-right corner and select **Settings**.
    *   In the left sidebar, click on **Developer settings**.
    *   Click on **OAuth Apps**, then click the **New OAuth App** button.

2.  **Fill in the Application Details:**
    *   **Application name:** A descriptive name for your application (e.g., "Decap CMS Auth").
    *   **Homepage URL:** The URL of your website or Decap CMS instance.
    *   **Authorization callback URL:** This is the URL where users will be redirected after they authorize the application. It should be in the format `https://<your-oauth-provider-domain>/callback`. For example, if you are running this service at `https://oauth.example.com`, the callback URL will be `https://oauth.example.com/callback`.

3.  **Generate Client ID and Secret:**
    *   After creating the application, you will be redirected to its page.
    *   You will see your **Client ID**. Copy this value and set it as the `OAUTH_CLIENT_ID` environment variable.
    *   Click the **Generate a new client secret** button. Copy the generated secret and set it as the `OAUTH_SECRET` environment variable. **Note:** This secret is only displayed once, so make sure to save it securely.

## Running with Docker

To run the application using Docker, first build the image:

```shell
docker build -t decap-cms-oauth .
```

Then, run the container with the required environment variables:

```shell
docker run \
  -e OAUTH_CLIENT_ID=<your_client_id> \
  -e OAUTH_SECRET=<your_secret> \
  -e OAUTH_ORIGINS=<your_origins> \
  -p 8080:3005 \
  decap-cms-oauth
```

The application will be available at `http://localhost:8080`.

### Using the Pre-built Image

Alternatively, you can pull the pre-built image from the GitHub Container Registry:

```shell
docker pull ghcr.io/blackb1rd/decap-cms-oauth:latest
```

Then, run the container:

```shell
docker run \
  -e OAUTH_CLIENT_ID=<your_client_id> \
  -e OAUTH_SECRET=<your_secret> \
  -e OAUTH_ORIGINS=<your_origins> \
  -p 8080:3005 \
  ghcr.io/blackb1rd/decap-cms-oauth:latest
```

## Using a Reverse Proxy (Nginx)

In a production environment, it is recommended to run the application behind a reverse proxy like Nginx. This allows you to handle SSL termination, caching, and other advanced features.

Here is a sample Nginx configuration for proxying requests to the application:

```nginx
server {
    listen 80;
    server_name your_domain.com;

    # Redirect HTTP to HTTPS
    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl http2;
    server_name your_domain.com;

    # SSL certificate configuration
    ssl_certificate /path/to/your/fullchain.pem;
    ssl_certificate_key /path/to/your/privkey.pem;

    location / {
        proxy_pass http://localhost:3005;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

This configuration listens for both HTTP and HTTPS traffic. HTTP traffic is redirected to HTTPS. The `proxy_pass` directive forwards requests to the application running on `localhost:3005`.

## Decap CMS Configuration

To use this provider with Decap CMS, you need to configure the `backend` in your `config.yml` file. Set `base_url` to the URL of your deployed provider and `auth_endpoint` to `auth`.

```yaml
backend:
  name: github
  repo: your-org/your-repo
  base_url: https://your-oauth-provider.com
  auth_endpoint: auth
```
