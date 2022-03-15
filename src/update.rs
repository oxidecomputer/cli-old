use std::fs;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::config_file::get_env_var;

/// ReleaseInfo stores information about a release.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReleaseInfo {
    #[serde(rename = "tag_name")]
    pub version: String,
    pub url: String,
    pub published_at: chrono::DateTime<chrono::Utc>,
}

/// StateEntry stores information about a state.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StateEntry {
    pub checked_for_update_at: chrono::DateTime<chrono::Utc>,
    pub latest_release: ReleaseInfo,
}

/// Check for updates to the cli.
///
/// Returns the latest version of the cli, or none if there is not a new
/// update or we shouldn't update.
pub async fn check_for_update(current_version: &str) -> Result<Option<ReleaseInfo>> {
    if !should_check_for_update() {
        return Ok(None);
    }

    let state_file = crate::config_file::state_file()?;

    // Get our current state.
    if std::path::Path::new(&state_file).exists() {
        let state = get_state_entry(&state_file)?;

        let duration_since_last_check = chrono::Utc::now() - state.checked_for_update_at;
        if duration_since_last_check < chrono::Duration::hours(24) {
            // If we've checked for updates in the last 24 hours, don't check again.
            return Ok(None);
        }
    }

    // Get the latest release.
    let latest_release = get_latest_release_info().await?;

    // Update our state.
    set_state_entry(&state_file, chrono::Utc::now(), latest_release.clone())?;

    if version_greater_then(&latest_release.version, current_version)? {
        return Ok(Some(latest_release));
    }

    Ok(None)
}

/// If we should check for an update to the cli.
fn should_check_for_update() -> bool {
    if !get_env_var("KITTYCAD_NO_UPDATE_NOTIFIER").is_empty() {
        return false;
    }

    !is_ci() && atty::is(atty::Stream::Stdout) && atty::is(atty::Stream::Stderr)
}

/// If we are running in a CI environment.
fn is_ci() -> bool {
    !get_env_var("CI").is_empty() || // GitHub Actions, Travis CI, CircleCI, Cirrus CI, GitLab CI, CommandVeyor, CodeShip, dsari
		!get_env_var("BUILD_NUMBER").is_empty() || // Jenkins, TeamCity
		!get_env_var("RUN_ID").is_empty() // TaskCluster, dsari
}

/// Get the information about the latest version of the cli.
async fn get_latest_release_info() -> Result<ReleaseInfo> {
    // If the user has a GITHUB_TOKEN environment variable, use it to get the latest release.
    // This allows us to test this while the repo is still private.
    // We might want to remove this in the future.
    let github_token = crate::config_file::get_env_var("GITHUB_TOKEN");

    let url = "https://api.github.com/repos/oxidecomputer/cli/releases/latest";

    let mut req = reqwest::Client::new().get(url);

    // Set the user agent.
    req = req.header("User-Agent", format!("oxide/{}", clap::crate_version!()));

    if !github_token.is_empty() {
        req = req.bearer_auth(github_token);
    }

    let resp = req.send().await?;
    let text = resp.text().await?;

    let latest_release: ReleaseInfo = match serde_json::from_str(&text) {
        Ok(release_info) => release_info,
        Err(err) => {
            return Err(anyhow!(
                "Failed to parse response from GitHub: {}\ntext:\n{}",
                err.to_string(),
                text
            ));
        }
    };

    Ok(latest_release)
}

/// Get an entry in the state file.
fn get_state_entry(filepath: &str) -> Result<StateEntry> {
    let file_content = fs::read_to_string(filepath)?;
    let state_entry: StateEntry = toml::from_str(&file_content)?;

    Ok(state_entry)
}

/// Set an entry in the state file.
fn set_state_entry(filename: &str, t: chrono::DateTime<chrono::Utc>, r: ReleaseInfo) -> Result<()> {
    let data = StateEntry {
        checked_for_update_at: t,
        latest_release: r,
    };

    let content = toml::to_string(&data)?;

    // Make sure we have a parent directory.
    let path = std::path::Path::new(&filename);
    let parent = path.parent().unwrap();
    fs::create_dir_all(parent).with_context(|| format!("failed to create directory {}", parent.display()))?;

    // Write the file.
    fs::write(filename, content).with_context(|| format!("failed to write file {}", filename))?;

    Ok(())
}

/// Return is one version is greater than another.
fn version_greater_then(v: &str, w: &str) -> Result<bool> {
    match version_compare::compare(v, w) {
        Ok(cmp) => Ok(cmp == version_compare::Cmp::Gt),
        Err(_) => Err(anyhow!("failed to compare versions: {} {}", v, w)),
    }
}

/// Returns if the release was published in the last 24 hours.
pub fn is_recent_release(published_at: chrono::DateTime<chrono::Utc>) -> bool {
    let duration = chrono::Utc::now() - published_at;

    duration.num_days() < 1
}

/// Check whether the oxide binary was found under the Homebrew prefix.
pub fn is_under_homebrew() -> Result<bool> {
    let binary_path = std::env::current_exe()?;
    let binary_path_str = binary_path.to_str().unwrap();

    let output = std::process::Command::new("brew").args(vec!["--prefix"]).output()?;

    let homebrew_prefix = String::from_utf8(output.stdout)?;

    let brew_bin_prefix = std::path::Path::new(homebrew_prefix.trim()).join("bin");

    Ok(binary_path_str.starts_with(brew_bin_prefix.to_str().unwrap()))
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    pub struct TestItem {
        name: String,
        current_version: String,
        latest_version: String,
        want_result: bool,
    }

    #[test]
    fn test_update() {
        let tests: Vec<TestItem> = vec![
            TestItem {
                name: "latest is newer".to_string(),
                current_version: "v0.0.1".to_string(),
                latest_version: "v1.0.0".to_string(),
                want_result: true,
            },
            TestItem {
                name: "current is prerelease".to_string(),
                current_version: "v1.0.0-pre.1".to_string(),
                latest_version: "v1.0.0".to_string(),
                want_result: true,
            },
            TestItem {
                name: "current is built from source".to_string(),
                current_version: "v1.2.3-123-gdeadbeef".to_string(),
                latest_version: "v1.2.3".to_string(),
                want_result: false,
            },
            TestItem {
                name: "current is built from source after a prerelease".to_string(),
                current_version: "v1.2.3-rc.1-123-gdeadbeef".to_string(),
                latest_version: "v1.2.3".to_string(),
                want_result: true,
            },
            TestItem {
                name: "latest is newer than version build from source".to_string(),
                current_version: "v1.2.3-123-gdeadbeef".to_string(),
                latest_version: "v1.2.4".to_string(),
                want_result: true,
            },
            TestItem {
                name: "latest is current".to_string(),
                current_version: "v1.2.5".to_string(),
                latest_version: "v1.2.5".to_string(),
                want_result: false,
            },
            TestItem {
                name: "latest is older".to_string(),
                current_version: "v0.10.0-pre.1".to_string(),
                latest_version: "v0.9.0".to_string(),
                want_result: false,
            },
        ];

        for t in tests {
            let result = crate::update::version_greater_then(&t.latest_version, &t.current_version).unwrap();

            assert_eq!(result, t.want_result, "test {} failed", t.name);
        }
    }
}
