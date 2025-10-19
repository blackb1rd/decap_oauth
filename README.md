External OAuth provider for Decap CMS. The following environment variables must be set for it to
work:

```shell
OAUTH_CLIENT_ID=(insert_the_client_id)
OAUTH_SECRET=(insert_the_secret)
OAUTH_ORIGINS=www.example.com,oauth.mysite.com
```

When using GitHub Enterprise, please set `OAUTH_HOSTNAME` to the proper value.

Documentation available on [docs.rs](https://docs.rs/decap_oauth/latest/decap_oauth/).
