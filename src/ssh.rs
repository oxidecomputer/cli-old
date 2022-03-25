use anyhow::Result;
use std::io::BufRead;

/// Retrieve the public ssh keys for a specific github user.
pub async fn get_github_ssh_keys(gh_handle: &str) -> Result<Vec<sshkeys::PublicKey>> {
    let resp = reqwest::get(&format!("https://github.com/{}.keys", gh_handle)).await?;
    let body = resp.bytes().await?;

    let reader = std::io::BufReader::new(body.as_ref());
    let lines: Vec<_> = reader.lines().collect();

    let mut keys: Vec<sshkeys::PublicKey> = Vec::new();
    for l in lines {
        let line = l?;
        // Parse the key.
        let key = sshkeys::PublicKey::from_string(&line)?;

        // Add the key to the list.
        keys.push(key);
    }

    Ok(keys)
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn test_get_github_ssh_keys() {
        let result = super::get_github_ssh_keys("jessfraz").await;
        assert_eq!(result.is_ok(), true);

        let keys = result.unwrap();

        assert_eq!(keys.len() > 0, true);
    }
}
