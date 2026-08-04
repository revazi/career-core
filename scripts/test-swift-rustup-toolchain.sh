#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TEMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TEMP_DIR"' EXIT

HOMEBREW_BIN="$TEMP_DIR/homebrew/bin"
RUSTUP_BIN="$TEMP_DIR/rustup/bin"
TOOLCHAIN_BIN="$TEMP_DIR/rustup/toolchains/stable/bin"
mkdir -p "$HOMEBREW_BIN" "$RUSTUP_BIN" "$TOOLCHAIN_BIN"

for command in cargo rustc; do
  cat > "$HOMEBREW_BIN/$command" <<'SH'
#!/usr/bin/env bash
printf 'unexpected Homebrew executable invocation\n' >&2
exit 99
SH
  cat > "$TOOLCHAIN_BIN/$command" <<'SH'
#!/usr/bin/env bash
if [[ "${1:-}" == '--print' && "${2:-}" == 'target-libdir' ]]; then
  printf '%s\n' "${FAKE_RUSTUP_TARGET_LIBDIR:-/missing-rust-target}"
fi
SH
  chmod +x "$HOMEBREW_BIN/$command" "$TOOLCHAIN_BIN/$command"
done

cat > "$RUSTUP_BIN/rustup" <<'SH'
#!/usr/bin/env bash
if [[ "${FAKE_RUSTUP_MISSING_TOOLCHAIN:-}" == '1' ]]; then
  exit 1
fi
case "${1:-}:${2:-}:${3:-}:${4:-}" in
  which:cargo:--toolchain:*) printf '%s\n' "$FAKE_RUSTUP_CARGO" ;;
  which:rustc:--toolchain:*) printf '%s\n' "$FAKE_RUSTUP_RUSTC" ;;
  show:active-toolchain:*) printf '%s\n' stable ;;
  *) exit 1 ;;
esac
SH
chmod +x "$RUSTUP_BIN/rustup"

assert_contains() {
  local haystack="$1"
  local needle="$2"
  if [[ "$haystack" != *"$needle"* ]]; then
    printf 'missing expected output: %s\n' "$needle" >&2
    printf 'actual output:\n%s\n' "$haystack" >&2
    exit 1
  fi
}

selection_output="$({
  PATH="$HOMEBREW_BIN:$RUSTUP_BIN:$PATH" \
  RUSTUP_TOOLCHAIN=stable \
  FAKE_RUSTUP_CARGO="$TOOLCHAIN_BIN/cargo" \
  FAKE_RUSTUP_RUSTC="$TOOLCHAIN_BIN/rustc" \
  /bin/bash -c '
    source "$1/scripts/swift-rustup-toolchain.sh"
    career_swift_select_rustup_toolchain
    printf "cargo=%s\\nrustc=%s\\ntoolchain=%s\\n" \
      "$CAREER_SWIFT_CARGO" "$CAREER_SWIFT_RUSTC" "$CAREER_SWIFT_TOOLCHAIN"
  ' -- "$ROOT_DIR"
} 2>&1)"
assert_contains "$selection_output" "cargo=$TOOLCHAIN_BIN/cargo"
assert_contains "$selection_output" "rustc=$TOOLCHAIN_BIN/rustc"
assert_contains "$selection_output" 'toolchain=stable'
if [[ "$selection_output" == *'unexpected Homebrew executable invocation'* ]]; then
  printf 'Homebrew-first PATH bypassed rustup selection\n' >&2
  exit 1
fi

target_failure_output=''
if target_failure_output="$(PATH="$HOMEBREW_BIN:$RUSTUP_BIN:$PATH" \
  RUSTUP_TOOLCHAIN=stable \
  FAKE_RUSTUP_CARGO="$TOOLCHAIN_BIN/cargo" \
  FAKE_RUSTUP_RUSTC="$TOOLCHAIN_BIN/rustc" \
  FAKE_RUSTUP_TARGET_LIBDIR="$TEMP_DIR/missing-target" \
  /bin/bash -c '
    source "$1/scripts/swift-rustup-toolchain.sh"
    career_swift_select_rustup_toolchain
    career_swift_require_rust_target aarch64-apple-ios
  ' -- "$ROOT_DIR" 2>&1)"; then
  printf 'missing Rust target unexpectedly passed\n' >&2
  exit 1
fi
assert_contains "$target_failure_output" 'error: Rust target is not installed: aarch64-apple-ios'
assert_contains "$target_failure_output" "selected cargo executable: $TOOLCHAIN_BIN/cargo"
assert_contains "$target_failure_output" 'selected toolchain: stable'
assert_contains "$target_failure_output" 'Install it with: rustup target add --toolchain stable aarch64-apple-ios'

toolchain_failure_output=''
if toolchain_failure_output="$(PATH="$HOMEBREW_BIN:$RUSTUP_BIN:$PATH" \
  RUSTUP_TOOLCHAIN=stable \
  FAKE_RUSTUP_MISSING_TOOLCHAIN=1 \
  /bin/bash -c '
    source "$1/scripts/swift-rustup-toolchain.sh"
    career_swift_select_rustup_toolchain
  ' -- "$ROOT_DIR" 2>&1)"; then
  printf 'missing rustup toolchain unexpectedly passed\n' >&2
  exit 1
fi
assert_contains "$toolchain_failure_output" 'error: rustup toolchain is unavailable: stable'
assert_contains "$toolchain_failure_output" 'selected toolchain: stable'
assert_contains "$toolchain_failure_output" 'selected cargo executable: unavailable'
assert_contains "$toolchain_failure_output" 'Install it with: rustup toolchain install stable'

invalid_toolchain='PRIVATE_TOOLCHAIN_TOKEN/invalid'
invalid_toolchain_output=''
if invalid_toolchain_output="$(PATH="$HOMEBREW_BIN:$RUSTUP_BIN:$PATH" \
  RUSTUP_TOOLCHAIN="$invalid_toolchain" \
  /bin/bash -c '
    source "$1/scripts/swift-rustup-toolchain.sh"
    career_swift_select_rustup_toolchain
  ' -- "$ROOT_DIR" 2>&1)"; then
  printf 'invalid rustup toolchain unexpectedly passed\n' >&2
  exit 1
fi
assert_contains "$invalid_toolchain_output" 'error: RUSTUP_TOOLCHAIN must be a bounded rustup toolchain name.'
assert_contains "$invalid_toolchain_output" 'selected toolchain: unavailable'
assert_contains "$invalid_toolchain_output" 'Install a toolchain with: rustup toolchain install stable'
if [[ "$invalid_toolchain_output" == *"$invalid_toolchain"* ]]; then
  printf 'invalid rustup toolchain leaked into diagnostics\n' >&2
  exit 1
fi

rustup_failure_output=''
if rustup_failure_output="$(PATH="$HOMEBREW_BIN" /bin/bash -c '
  source "$1/scripts/swift-rustup-toolchain.sh"
  career_swift_select_rustup_toolchain
' -- "$ROOT_DIR" 2>&1)"; then
  printf 'missing rustup unexpectedly passed\n' >&2
  exit 1
fi
assert_contains "$rustup_failure_output" 'error: rustup is required to select the Rust toolchain for Swift artifacts.'
assert_contains "$rustup_failure_output" 'selected rustup executable: unavailable'
assert_contains "$rustup_failure_output" "Install it with: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"

printf 'Swift rustup toolchain guards passed.\n'
