External OAuth provider for Decap CMS. The following environment variables must be set for it to
work:

```shell
OAUTH_CLIENT_ID=(insert_the_client_id)
OAUTH_SECRET=(insert_the_secret)
OAUTH_ORIGINS=www.example.com,oauth.mysite.com
```

When using GitHub Enterprise, please set `OAUTH_HOSTNAME` to the proper value.

## Docker Usage

### Prerequisites

Before building the Docker image, you need to vendor the Rust dependencies. You can do this manually or use the provided script:

**Option 1: Using the preparation script (recommended)**
```bash
./prepare-docker.sh
```

**Option 2: Manual preparation**
```bash
cargo vendor
```

This creates a `vendor` directory with all dependencies, which is required for the Docker build.

### Using Docker Compose (Recommended)

1. Copy `.env.example` to `.env`:
   ```bash
   cp .env.example .env
   ```

2. Edit `.env` file with your OAuth credentials:
   ```shell
   OAUTH_CLIENT_ID=your_github_oauth_client_id
   OAUTH_SECRET=your_github_oauth_secret
   OAUTH_ORIGINS=www.example.com,oauth.mysite.com
   ```

3. Start the service:
   ```bash
   docker-compose up -d
   ```

4. View logs:
   ```bash
   docker-compose logs -f
   ```

5. Stop the service:
   ```bash
   docker-compose down
   ```

### Using Docker Directly

1. Build the Docker image:
   ```bash
   docker build -t decap-oauth .
   ```

2. Run the container:
   ```bash
   docker run -d \
     -p 3005:3005 \
     -e OAUTH_CLIENT_ID=your_client_id \
     -e OAUTH_SECRET=your_secret \
     -e OAUTH_ORIGINS=www.example.com \
     --name decap-oauth \
     decap-oauth
   ```

The server will be available at `http://localhost:3005`.

Documentation available on [docs.rs](https://docs.rs/decap_oauth/latest/decap_oauth/).
