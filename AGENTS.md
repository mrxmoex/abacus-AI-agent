# Working in this repository

Abacus is a terminal coding agent: one Rust binary named `abacus`, built from
`src/` with integration tests in `tests/`. There is no service to start and no
database to provision.

## Requirements

Rust 1.88 or newer, because the crate uses edition 2024. `cargo --version`
should report at least 1.88; CI enforces that floor with a dedicated MSRV job.

## Validating a change

Use `scripts/validate.sh`. It stops at the first failure and exits with that
command's status.

```sh
./scripts/validate.sh          # quick: fmt check and cargo check
./scripts/validate.sh smoke    # quick, plus the CLI test that needs no API key
./scripts/validate.sh full     # fmt, clippy, and the whole suite, matching CI
```

`quick` is the fast edit loop. Run `full` before pushing; it is what CI runs.
Add `--dry-run` to print the commands without running them.

`git config core.hooksPath .githooks` opts a clone into the pre-push hook,
which runs `quick` before every push. Bypass once with `git push --no-verify`.

For a single area, the underlying commands still work directly, for example
`cargo test --lib config::` or `cargo test --test e2e_agent`.

## Running the agent without an API account

`abacus setup` refuses to run without a terminal, so a non-interactive
environment must supply configuration another way. Both paths below work with no
provider account.

Point the binary at any OpenAI-compatible endpoint on the command line:

```sh
ABACUS_NO_ACTIVITY=1 abacus \
  --base-url http://127.0.0.1:11434/v1 \
  --model your-tool-capable-model \
  --protocol chat-completions \
  --no-session \
  -p "Explain this repository"
```

Or write `$ABACUS_HOME/config.toml` (default `~/.abacus/config.toml`) directly:

```toml
version = 2
default_profile = "local"

[profiles.local]
name = "Local"
base_url = "http://127.0.0.1:11434/v1"
model = "your-tool-capable-model"
protocol = "chat-completions"
```

Then `abacus doctor` reports whether the endpoint answers. When nothing listens
at the configured address, a headless run fails immediately with `provider
unreachable` and the path of the config file to fix, rather than hanging.

`tests/cli_headless.rs` shows the same path end to end against a local stub
server, which is why `smoke` needs no credentials.

## Environment variables

| Variable | Purpose |
| --- | --- |
| `OPENAI_API_KEY`, `XAI_API_KEY`, `OPENROUTER_API_KEY` | Provider keys, preferred over stored credentials |
| `ABACUS_API_KEY`, `ABACUS_BASE_URL`, `ABACUS_MODEL` | Override the resolved profile |
| `ABACUS_HOME` | Relocate state; point it at a temp directory in tests |
| `ABACUS_NO_ACTIVITY` | Set to `1` to disable anonymous activity reporting |

## Conventions

Add a regression test with any behavior change; `tests/` covers the agent loop,
providers, MCP, and the CLI, and unit tests live beside the code they cover.
Keep provider differences behind the OpenAI-compatible boundary in
`src/provider.rs`, and keep session and config formats forwards compatible.
Comments explain constraints, not mechanics.
