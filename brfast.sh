#!/usr/bin/env sh
set -eu

taskkill.exe /IM fixtext.exe /F >/dev/null 2>&1 || true
zig build -Doptimize=ReleaseSafe
./zig-out/bin/fixtext.exe >/dev/null 2>&1 &
