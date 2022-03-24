# cli

The Oxide command line tool.

The instructions below refer to instructions for contributing to the repo.

For the CLI docs for end users refer to: https://docs.oxide.computer/cli/manual

### Installing

Instructions for installing are on the [latest release](https://github.com/oxidecomputer/cli/releases).

### Updating the API spec

Updating the API spec is as simple as updating the [`spec.json`](spec.json) file. The macro will take it from there when
you `cargo build`. It likely might need some tender love and care to make it a nice command like the other generated ones
if it is out of the ordinary.

Only `create`, `edit`, `view/get`, `list`, `delete` commands are generated. The rest are bespoke and any generation lead to something
that seemed harder to maintain over time. But if you are brave you can try.

For examples of the macro formatting, checkout some of the commands under `src/` like `cmd_disk` or `cmd_org`.

**Note:** If you update the API spec here, you will likely want to bump the spec for the [oxide.rs](https://github.com/oxidecomputer/oxide.rs)
repo as well since that is where the API client comes from.

### Running the tests

The tests require a nexus server. The tests use the `OXIDE_TEST_TOKEN` and `OXIDE_TEST_HOST` variables for knowing where to look and authenticate.
You need to run the tests with `--test-threads=1`, ex: `cargo test -- --test-threads=1` because env variables are being set and tested and things will overlap.

For now the token for spoof is `oxide-spoof-001de000-05e4-4000-8000-000000004007`.

**Note:** you DON'T want to run the tests against your production account, since it will create a bunch of stuff and then destroy what it created (and likely everything else).
