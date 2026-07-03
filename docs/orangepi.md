# Orange Pi PC Setup

Brausi targets ARMv7 32-bit Linux boards such as the Orange Pi PC.

## Runtime Bootstrap

On Debian or Armbian:

```sh
sudo ./scripts/bootstrap-debian.sh
./scripts/check-deps.sh
```

Required runtime tools:

- `Xvfb`
- `chromium` or `chromium-browser`
- `scrot`
- `chafa`
- `xdotool`

Optional future mirror/cast tools:

- `ffmpeg`
- `cvlc`

Install optional mirror/cast tools with:

```sh
sudo BRAUSI_BOOTSTRAP_OPTIONAL=1 ./scripts/bootstrap-debian.sh
```

## Build Target

The expected Rust target is:

```text
armv7-unknown-linux-gnueabihf
```

The first implementation has no Rust crate dependencies, so it can be built on
the board if Rust is installed:

```sh
cargo build --release
```

For appliance-style deployment:

```sh
sudo install -m 0755 target/release/brausi /usr/local/bin/brausi
sudo mkdir -p /var/lib/brausi /var/log/brausi
```

Then run with explicit state paths:

```sh
BRAUSI_STATE_DIR=/var/lib/brausi brausi start https://example.com
BRAUSI_STATE_DIR=/var/lib/brausi brausi view
```

## Chromium Path

If the distro package installs `chromium-browser` instead of `chromium`, use:

```sh
BRAUSI_CHROMIUM=chromium-browser brausi start https://example.com
```

## First Smoke Test

```sh
brausi start https://example.com
brausi status
brausi view
```

If the rendered image wraps across lines, force a smaller terminal render size:

```sh
BRAUSI_RENDER_COLS=79 BRAUSI_RENDER_LINES=23 brausi view
```

In another terminal:

```sh
brausi click 320 240
brausi go https://lite.duckduckgo.com/lite/
brausi stop
```
