> **Warning**
>
> This CLI is no longer supported and will not be maintained.
> Refer to https://github.com/oxidecomputer/oxide-sdk-and-cli for the current CLI and Rust client

# cli

The Oxide command line tool.

The instructions below refer to instructions for contributing to the repo.

For the CLI docs for end users refer to: https://docs.oxide.computer/cli

If you are running nexus locally without `https://` make sure you denote that in
the URL you pass to `OXIDE_HOST` or to `oxide auth login`.

### Authentication

To authenticate today, you can use the spoof token:
`oxide-spoof-001de000-05e4-4000-8000-000000004007`

You can get a non-spoof access token with `oxide auth login`.
That will contact `OXIDE_HOST` and attempt an OAuth 2.0 Device
Authorization Grant. The CLI will attempt to open a browser window
with which you can login (via SAML or other IdP method) and type in
or verify the user code printed in the terminal. After a successful
login and code verification, a token associated with the logged-in
user will be granted and stored in the config file.

### Installing

Instructions for installing are on the [latest release](https://github.com/oxidecomputer/cli/releases).

### Updating the API spec

Updating the API spec is as simple as updating the [`spec.json`](spec.json) file. The macro will take it from there when
you `cargo build`. It likely might need some tender love and care to make it a nice command like the other generated ones
if it is out of the ordinary.

**Important: Currently we are transitioning to use progenitor as a client generator instead of the current client generator.**
**This means that as a temporary work around the spec.json file must be copied from oxide.rs and you must make sure all tags don't change**

Only `create`, `edit`, `view/get`, `list`, `delete` commands are generated. The rest are bespoke and any generation lead to something
that seemed harder to maintain over time. But if you are brave you can try.

For examples of the macro formatting, checkout some of the commands under `src/` like `cmd_disk` or `cmd_org`.

**Note:** If you update the API spec here, you will likely want to bump the spec for the [oxide.rs](https://github.com/oxidecomputer/oxide.rs)
repo as well since that is where the API client comes from.

### Running the tests

The tests require a nexus server. The tests use the `OXIDE_TEST_TOKEN` and `OXIDE_TEST_HOST` variables for knowing where to look and authenticate.

For now the token for spoof is `oxide-spoof-001de000-05e4-4000-8000-000000004007`.

**Note:** you DON'T want to run the tests against your production account, since it will create a bunch of stuff and then destroy what it created (and likely everything else).

### Releasing a new version

1. Make sure the `Cargo.toml` has the new version you want to release.
2. Run `make tag` this is just an easy command for making a tag formatted
   correctly with the version.
3. Push the tag (the result of `make tag` gives instructions for this)
4. Everything else is triggered from the tag push. Just make sure all the tests
   and cross compilation pass on the `main` branch before making and pushing
   a new tag.

### Building

To build, simply run `cargo build` like usual.

Make sure to update to the latest stable rustc: for example, if you use `rustup`, run `rustup update`.

#### Cross compiling

If you're on Debian or Ubuntu, install the required dependencies by running `.github/workflows/cross-deps.sh`. Otherwise, look there to see what packages are required.

Then, simply run `make`. Binaries will be available in `cross/`.

If you want to only build one of the cross targets, supply the `CROSS_TARGETS` environment variable:

    CROSS_TARGETS=x86_64-unknown-linux-musl make

### Docs

The data powering the CLI docs at [docs.oxide.computer/cli](https://docs.oxide.computer/cli) is produced by the `oxide generate` CLI command. This command takes a positional argument specifying the output format. The options are `json`, `markdown`, and `man-pages`. `json` produces a single file, while `markdown` and `man-pages` produce a file for every command and subcommand.

We version a copy of the generated JSON in this repo at [docs/oxide.json](docs/oxide.json) so any revision can be fetched by the docs site at build time. The test `test_generate_json` will fail if `docs/oxide.json` has not been updated with the CLI changes on a given branch. To update the file, run `cargo run -- generate json -D docs` or run `test_generate_json` with `EXPECTORATE=overwrite` set.
