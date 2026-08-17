#!/usr/bin/env sh
# Repository validation entrypoint.
#
#   scripts/validate.sh [quick|smoke|full] [--dry-run]
#
#   quick  formatting and compilation only
#   smoke  quick, plus the headless CLI test that needs no API key
#   full   every gate CI enforces
set -eu

mode=quick
dry_run=0

usage() {
    cat <<'EOF'
Usage: scripts/validate.sh [MODE] [--dry-run]

Modes:
  quick   cargo fmt check and cargo check (default)
  smoke   quick, plus the headless CLI test that needs no API key
  full    fmt, clippy, and the whole test suite, matching CI
EOF
}

for argument in "$@"; do
    case "$argument" in
        quick | smoke | full) mode=$argument ;;
        --dry-run) dry_run=1 ;;
        -h | --help)
            usage
            exit 0
            ;;
        *)
            printf 'validate: unknown argument "%s"\n\n' "$argument" >&2
            usage >&2
            exit 2
            ;;
    esac
done

# Keep local runs off the activity service, exactly as CI does.
ABACUS_NO_ACTIVITY=1
export ABACUS_NO_ACTIVITY

step() {
    if [ "$dry_run" -eq 1 ]; then
        printf '%s\n' "$1"
        return 0
    fi
    printf '\n==> %s\n' "$1"
    status=0
    sh -c "$1" || status=$?
    if [ "$status" -ne 0 ]; then
        printf '\nvalidate: %s failed (%s)\n' "$mode" "$1" >&2
        exit "$status"
    fi
}

step 'cargo fmt --all -- --check'

case "$mode" in
    quick)
        step 'cargo check --all-targets --locked'
        ;;
    smoke)
        step 'cargo check --all-targets --locked'
        step 'cargo test --locked --test cli_headless'
        ;;
    full)
        step 'cargo clippy --all-targets --locked -- -D warnings'
        step 'cargo test --all-targets --locked'
        ;;
esac

if [ "$dry_run" -eq 0 ]; then
    printf '\nvalidate: %s passed\n' "$mode"
fi
