use std::fs;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::config_file::get_env_var;

/// ReleaseInfo stores information about a release.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReleaseInfo {
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
fn check_for_update(_current_version: &str) -> Result<Option<ReleaseInfo>> {
    if !should_check_for_update() {
        return Ok(None);
    }

    // TODO: Fill in here.

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
    !get_env_var("CI").is_empty() || // GitHub Actions, Travis CI, CircleCI, Cirrus CI, GitLab CI, AppVeyor, CodeShip, dsari
		!get_env_var("BUILD_NUMBER").is_empty() || // Jenkins, TeamCity
		!get_env_var("RUN_ID").is_empty() // TaskCluster, dsari
}

/// Get the information about the latest version of the cli.
async fn get_latest_release_info() -> Result<ReleaseInfo> {
    let latest_release: ReleaseInfo = reqwest::get("https://api.github.com/repos/oxidecomputer/cli/releases/latest")
        .await?
        .json()
        .await?;

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
fn version_greater_then(v: &str, w: &str) -> bool {
    let cmp = version_compare::compare(v, w).unwrap_or(version_compare::Cmp::Eq);

    cmp == version_compare::Cmp::Gt
}

/// Returns if the release was published in the last 24 hours.
fn is_recent_release(published_at: chrono::DateTime<chrono::Utc>) -> bool {
    let duration = chrono::Utc::now() - published_at;

    duration.num_days() < 1
}

/// Check whether the oxide binary was found under the Homebrew prefix.
fn is_under_homebrew(binary_path: &str) -> Result<bool> {
    let output = std::process::Command::new("brew").args(vec!["--prefix"]).output()?;

    let homebrew_prefix = String::from_utf8(output.stdout)?;

    let brew_bin_prefix = std::path::Path::new(homebrew_prefix.trim()).join("bin");

    Ok(binary_path.starts_with(brew_bin_prefix.to_str().unwrap()))
}
