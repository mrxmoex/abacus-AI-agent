# Contributing

Abacus favors a small, inspectable core. New features should strengthen the coding loop, setup, safety, portability, or reliability without adding a required service.

Before submitting a change:

```sh
./scripts/validate.sh full
```

That runs the same gates as CI:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

While iterating, `./scripts/validate.sh` checks formatting and compilation only,
and `./scripts/validate.sh smoke` adds the headless CLI test, which needs no API
key. See [AGENTS.md](AGENTS.md) for the non-interactive setup path.

`git config core.hooksPath .githooks` opts your clone into the pre-push hook,
which runs the quick checks before every push (`git push --no-verify` skips it
once).

Add regression tests for behavior changes. Keep provider-specific behavior behind the existing OpenAI-compatible boundary where possible, and preserve session/config forwards compatibility.
