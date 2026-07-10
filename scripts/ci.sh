#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

for marker in Desktop Documents Downloads OneDrive; do
    case "$ROOT_DIR" in
        */"$marker"/*)
            WINDOWS_HOME="${ROOT_DIR%%/"$marker"/*}"
            [ -d "$WINDOWS_HOME/.cargo/bin" ] && export PATH="$WINDOWS_HOME/.cargo/bin:$PATH"
            [ -d "$WINDOWS_HOME/.bun/bin" ] && export PATH="$WINDOWS_HOME/.bun/bin:$PATH"
            ;;
    esac
done

if ! command -v cargo >/dev/null 2>&1; then
    if [ -d "$HOME/.cargo/bin" ]; then
        export PATH="$HOME/.cargo/bin:$PATH"
    fi
    if command -v cygpath >/dev/null 2>&1 && [ -n "${USERPROFILE:-}" ]; then
        WINDOWS_HOME="$(cygpath -u "$USERPROFILE")"
        export PATH="$WINDOWS_HOME/.cargo/bin:$PATH"
    fi
fi

if ! command -v bun >/dev/null 2>&1; then
    if [ -d "$HOME/.bun/bin" ]; then
        export PATH="$HOME/.bun/bin:$PATH"
    fi
    if command -v cygpath >/dev/null 2>&1 && [ -n "${USERPROFILE:-}" ]; then
        WINDOWS_HOME="$(cygpath -u "$USERPROFILE")"
        export PATH="$WINDOWS_HOME/.bun/bin:$PATH"
    fi
fi

if ! command -v cargo >/dev/null 2>&1; then
    echo "cargo no esta disponible en este entorno." >&2
    exit 127
fi

if ! command -v bun >/dev/null 2>&1; then
    echo "bun debe estar disponible para ejecutar la validacion completa." >&2
    exit 127
fi

bun install --frozen-lockfile
bun run fmt:check
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo build --all-targets --locked
cargo test --locked
bun run test:node
