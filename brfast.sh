#!/usr/bin/env sh
set -eu

now_ms() {
  timestamp="$(date +%s%3N 2>/dev/null || true)"
  case "$timestamp" in
  '' | *[!0-9]*)
    powershell.exe -NoProfile -Command "[DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()" | tr -d '\r'
    ;;
  *)
    printf '%s\n' "$timestamp"
    ;;
  esac
}

timed() {
  timed_label="$1"
  shift
  timed_start="$(now_ms)"
  printf '[brfast] %s...\n' "$timed_label"
  "$@"
  timed_end="$(now_ms)"
  printf '[brfast] %s: %sms\n' "$timed_label" "$((timed_end - timed_start))"
}

timed_bg() {
  timed_bg_label="$1"
  shift
  (
    timed_bg_start="$(now_ms)"
    printf '[brfast] %s...\n' "$timed_bg_label"
    "$@"
    timed_bg_end="$(now_ms)"
    printf '[brfast] %s: %sms\n' "$timed_bg_label" "$((timed_bg_end - timed_bg_start))"
  ) &
  LAST_PID="$!"
}

inputs_newer_than() {
  stamp_file="$1"
  shift
  [ -f "$stamp_file" ] || return 0
  find "$@" -newer "$stamp_file" -print -quit 2>/dev/null | grep -q .
}

ensure_frontend() {
  frontend_stamp="$cache_dir/frontend.stamp"
  if [ "${BRFAST_FORCE:-0}" != "1" ] && [ -f dist/index.html ] && ! inputs_newer_than "$frontend_stamp" \
    src \
    public \
    index.html \
    package.json \
    bun.lock \
    tsconfig.json \
    tsconfig.node.json \
    vite.config.ts; then
    printf '[brfast] frontend build: skipped (unchanged)\n'
  else
    timed "frontend build" bun run build:frontend:fast
    mkdir -p "$cache_dir"
    touch "$frontend_stamp"
  fi
}

ensure_tauri_executable() {
  rust_stamp="$cache_dir/rust-$mode.stamp"
  if [ "${BRFAST_FORCE:-0}" != "1" ] && [ -f "$exe_path" ] && ! inputs_newer_than "$rust_stamp" \
    dist \
    src-tauri/src \
    src-tauri/capabilities \
    src-tauri/build.rs \
    src-tauri/Cargo.toml \
    src-tauri/Cargo.lock \
    src-tauri/tauri.conf.json; then
    printf '[brfast] tauri executable build: skipped (unchanged)\n'
  else
    timed "tauri executable build" sh -c "$cargo_cmd"
    mkdir -p "$cache_dir"
    touch "$rust_stamp"
  fi
}

total_start="$(now_ms)"
mode="${BRFAST_MODE:-debug}"
if [ "$mode" = "release" ]; then
  cargo_cmd="cargo build --manifest-path src-tauri/Cargo.toml --release --features tauri/custom-protocol"
  exe_path="./src-tauri/target/release/fixtext.exe"
else
  cargo_cmd="cargo build --manifest-path src-tauri/Cargo.toml --features tauri/custom-protocol"
  exe_path="./src-tauri/target/debug/fixtext.exe"
fi
cache_dir="./src-tauri/target/brfast-v2"
run_log="./src-tauri/target/fixtext-run.log"
printf '[brfast] mode: %s\n' "$mode"

timed_bg "stop previous process" sh -c 'taskkill.exe /IM fixtext.exe /F >/dev/null 2>&1 || true'
stop_pid="$LAST_PID"
timed_bg "frontend build check" ensure_frontend
frontend_pid="$LAST_PID"

wait "$stop_pid"
wait "$frontend_pid"

ensure_tauri_executable
timed "launch app" sh -c "RUST_BACKTRACE=1 \"$exe_path\" >>\"$run_log\" 2>&1 &"

total_end="$(now_ms)"
printf '[brfast] total: %sms\n' "$((total_end - total_start))"
printf '[brfast] run log: %s\n' "$run_log"
