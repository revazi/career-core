#!/usr/bin/env bash
# Sourced by the Swift XCFramework build and verification scripts.

career_swift_print_toolchain_selection() {
  printf 'selected rustup executable: %s\n' "${CAREER_SWIFT_RUSTUP:-unavailable}" >&2
  printf 'selected toolchain: %s\n' "${CAREER_SWIFT_TOOLCHAIN:-unavailable}" >&2
  printf 'selected cargo executable: %s\n' "${CAREER_SWIFT_CARGO:-unavailable}" >&2
  printf 'selected rustc executable: %s\n' "${CAREER_SWIFT_RUSTC:-unavailable}" >&2
}

career_swift_toolchain_name_is_valid() {
  [[ "$1" =~ ^[A-Za-z0-9._+-]{1,128}$ ]]
}

career_swift_executable_path_is_valid() {
  [[ -n "$1" && ${#1} -le 512 && "$1" != *$'\n'* && -x "$1" ]]
}

career_swift_select_rustup_toolchain() {
  unset CAREER_SWIFT_RUSTUP CAREER_SWIFT_TOOLCHAIN CAREER_SWIFT_CARGO CAREER_SWIFT_RUSTC

  local rustup_path
  if ! rustup_path="$(command -v rustup)" \
    || ! career_swift_executable_path_is_valid "$rustup_path"; then
    printf 'error: rustup is required to select the Rust toolchain for Swift artifacts.\n' >&2
    career_swift_print_toolchain_selection
    printf "Install it with: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh\n" >&2
    return 2
  fi
  CAREER_SWIFT_RUSTUP="$rustup_path"

  local toolchain
  if [[ -n "${RUSTUP_TOOLCHAIN:-}" ]]; then
    toolchain="$RUSTUP_TOOLCHAIN"
  else
    local active_toolchain
    if ! active_toolchain="$("$CAREER_SWIFT_RUSTUP" show active-toolchain 2>/dev/null)"; then
      printf 'error: rustup has no active toolchain.\n' >&2
      career_swift_print_toolchain_selection
      printf 'Install it with: rustup toolchain install stable\n' >&2
      printf 'Select it with: rustup default stable\n' >&2
      return 2
    fi
    toolchain="${active_toolchain%% *}"
  fi

  if ! career_swift_toolchain_name_is_valid "$toolchain"; then
    printf 'error: RUSTUP_TOOLCHAIN must be a bounded rustup toolchain name.\n' >&2
    career_swift_print_toolchain_selection
    printf 'Install a toolchain with: rustup toolchain install stable\n' >&2
    return 2
  fi
  CAREER_SWIFT_TOOLCHAIN="$toolchain"

  local cargo_path
  local rustc_path
  if ! cargo_path="$("$CAREER_SWIFT_RUSTUP" which cargo --toolchain "$CAREER_SWIFT_TOOLCHAIN" 2>/dev/null)" \
    || ! career_swift_executable_path_is_valid "$cargo_path" \
    || ! rustc_path="$("$CAREER_SWIFT_RUSTUP" which rustc --toolchain "$CAREER_SWIFT_TOOLCHAIN" 2>/dev/null)" \
    || ! career_swift_executable_path_is_valid "$rustc_path"; then
    printf 'error: rustup toolchain is unavailable: %s\n' "$CAREER_SWIFT_TOOLCHAIN" >&2
    career_swift_print_toolchain_selection
    printf 'Install it with: rustup toolchain install %s\n' "$CAREER_SWIFT_TOOLCHAIN" >&2
    return 2
  fi

  CAREER_SWIFT_CARGO="$cargo_path"
  CAREER_SWIFT_RUSTC="$rustc_path"
  export RUSTC="$CAREER_SWIFT_RUSTC"
}

career_swift_require_rust_target() {
  local target="$1"
  local target_libdir
  if ! target_libdir="$("$CAREER_SWIFT_RUSTC" --print target-libdir --target "$target" 2>/dev/null)" \
    || [[ ! -d "$target_libdir" ]]; then
    printf 'error: Rust target is not installed: %s\n' "$target" >&2
    career_swift_print_toolchain_selection
    printf 'Install it with: rustup target add --toolchain %s %s\n' \
      "$CAREER_SWIFT_TOOLCHAIN" "$target" >&2
    return 2
  fi
}
