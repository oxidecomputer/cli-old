use anyhow::Result;

/// This function returns an API client for Oxide that is based on the configured
/// user..
fn new_api_client(ctx: &crate::context::Context) -> Result<oxide_api::Client> {
    // We need to get the default host from the config.
    let host = ctx.config.default_host()?;

    // Get the token for that host.
    let token = ctx.config.get(&host, "token")?;

    // Create the client.
    let mut client = oxide_api::Client::new(&token);

    // Change the baseURL to the one we want.
    let mut baseurl = format!("https://{}", host);
    if host.starts_with("localhost") {
        baseurl = format!("http://{}", host)
    }

    // Override the default host with the one from the config.
    client = client.with_host(baseurl);

    Ok(client)
}
