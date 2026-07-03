#!/usr/bin/env sh
set -eu

if [ "$(id -u)" -ne 0 ]; then
  echo "Run as root: sudo ./scripts/bootstrap-debian.sh" >&2
  exit 1
fi

INSTALL_OPTIONAL="${BRAUSI_BOOTSTRAP_OPTIONAL:-0}"
INSTALL_RUST="${BRAUSI_BOOTSTRAP_RUST:-0}"

apt_get_install() {
  DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends "$@"
}

echo "Updating package lists..."
apt-get update

echo "Installing Brausi runtime dependencies..."
apt_get_install \
  ca-certificates \
  xvfb \
  scrot \
  chafa \
  xdotool \
  fonts-dejavu-core

if apt-cache show chromium >/dev/null 2>&1; then
  apt_get_install chromium
elif apt-cache show chromium-browser >/dev/null 2>&1; then
  apt_get_install chromium-browser
else
  echo "Warning: neither chromium nor chromium-browser was found in apt." >&2
  echo "Install Chromium manually and set BRAUSI_CHROMIUM=/path/to/chromium." >&2
fi

if [ "$INSTALL_OPTIONAL" = "1" ]; then
  echo "Installing optional mirror/cast helpers..."
  apt_get_install ffmpeg vlc || true
fi

if [ "$INSTALL_RUST" = "1" ]; then
  echo "Installing distro Rust toolchain..."
  apt_get_install cargo rustc pkg-config build-essential
fi

echo
echo "Bootstrap complete. Verify with:"
echo "  ./scripts/check-deps.sh"
echo
echo "For Orange Pi appliance installs, copy the compiled binary to:"
echo "  /usr/local/bin/brausi"
