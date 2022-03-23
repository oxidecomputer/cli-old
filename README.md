# cli

The Oxide command line tool.

### Installing

Instructions for installing are on the [latest release](https://github.com/oxidecomputer/cli/releases).

### Updating the spec.

Updating the API spec is as simple as updating the [`spec.json`](spec.json) file. The macro will take it from there when
you `cargo build`. It likely might need some tender love and care to make it a nice command like the other generated ones
if it is out of the ordinary.

Only `create`, `edit`, `view/get`, `delete` commands are generated. The rest are bespoke and any generation lead to something
that seemed harder to maintain over time. But if you are brave you can try.
