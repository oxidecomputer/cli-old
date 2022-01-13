use crate::config_file::get_env_var;

/// If we should check for an update to the cli.
fn should_check_for_updates() -> bool {
    if get_env_var("KITTYCAD_NO_UPDATE_NOTIFIER") != "" {
        return false;
    }

    !is_ci() && atty::is(atty::Stream::Stdout) && atty::is(atty::Stream::Stderr)
}

/// If we are running in a CI environment.
fn is_ci() -> bool {
    get_env_var("CI") != "" || // GitHub Actions, Travis CI, CircleCI, Cirrus CI, GitLab CI, AppVeyor, CodeShip, dsari
		get_env_var("BUILD_NUMBER") != "" || // Jenkins, TeamCity
		get_env_var("RUN_ID") != "" // TaskCluster, dsari
}
