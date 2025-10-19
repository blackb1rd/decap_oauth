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

## Deployment

This repository includes a GitHub Actions workflow to automatically deploy the application to a server using Docker. The deployment is triggered on every push to the `main` branch.

### Server Setup

Ensure that Docker is installed on your server.

### Repository Configuration

To enable the deployment workflow, you must configure the following secrets in your GitHub repository's settings (`Settings > Secrets and variables > Actions`):

-   `SSH_PRIVATE_KEY`: The private SSH key used to connect to your server.
-   `SSH_USER_HOST`: The username and host of your server (e.g., `user@your-server.com`).
-   `OAUTH_CLIENT_ID`: The OAuth client ID for your application.
-   `OAUTH_SECRET`: The OAuth secret for your application.
-   `OAUTH_ORIGINS`: A comma-separated list of allowed origins.

The workflow will connect to your server via SSH, pull the latest Docker image, and then stop, remove, and restart the container. The `OAUTH_*` secrets are safely escaped and passed directly to the `docker run` command, ensuring that special characters are handled correctly.
