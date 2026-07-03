#!/usr/bin/env sh
set -eu

missing=0

check_cmd() {
  name="$1"
  required="${2:-1}"

  if command -v "$name" >/dev/null 2>&1; then
    printf 'ok      %s -> %s\n' "$name" "$(command -v "$name")"
    return 0
  fi

  if [ "$required" = "1" ]; then
    printf 'missing %s\n' "$name"
    missing=1
  else
    printf 'optional-missing %s\n' "$name"
  fi
}

printf 'Brausi dependency check\n'
printf '=======================\n'

check_cmd Xvfb
if command -v chromium >/dev/null 2>&1; then
  printf 'ok      chromium -> %s\n' "$(command -v chromium)"
elif command -v chromium-browser >/dev/null 2>&1; then
  printf 'ok      chromium-browser -> %s\n' "$(command -v chromium-browser)"
else
  printf 'missing chromium or chromium-browser\n'
  missing=1
fi
check_cmd scrot
check_cmd chafa
check_cmd xdotool
check_cmd ffmpeg 0
check_cmd cvlc 0
check_cmd rustc 0
check_cmd cargo 0

if [ "$missing" -ne 0 ]; then
  printf '\nRequired dependencies are missing. On Debian/Armbian, run:\n'
  printf '  sudo ./scripts/bootstrap-debian.sh\n'
  exit 1
fi

printf '\nRequired runtime dependencies are present.\n'
