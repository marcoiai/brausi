# Brausi

Brausi is a lightweight terminal browser controller. It does not implement a
browser engine. It starts Chromium in `Xvfb`, renders the virtual display into a
terminal with `scrot`/`chafa`, and sends input through `xdotool`.

## Commands

```sh
brausi start [url]
brausi view
brausi click <x> <y>
brausi move <x> <y>
brausi down <x> <y>
brausi up <x> <y>
brausi type <text>
brausi go <url>
brausi back
brausi forward
brausi reload
brausi status
brausi stop
```

Coordinates are framebuffer pixels. With the default display size, valid values
are `x=0..1023` and `y=0..767`.

## Local Build

```sh
cargo build --release
```

## Runtime Dependencies

Required:

- `Xvfb`
- `chromium` or `chromium-browser`
- `scrot`
- `chafa`
- `xdotool`

Check the current machine:

```sh
./scripts/check-deps.sh
```

Install on Debian/Armbian:

```sh
sudo ./scripts/bootstrap-debian.sh
```

Optional future mirror/cast helpers (`ffmpeg`, `vlc`) are opt-in:

```sh
sudo BRAUSI_BOOTSTRAP_OPTIONAL=1 ./scripts/bootstrap-debian.sh
```

## Configuration

Brausi reads simple environment variables:

```sh
BRAUSI_DISPLAY=:99
BRAUSI_WIDTH=1024
BRAUSI_HEIGHT=768
BRAUSI_STATE_DIR=~/.brausi
BRAUSI_CHROMIUM=chromium
BRAUSI_XVFB=Xvfb
BRAUSI_SCROT=scrot
BRAUSI_CHAFA=chafa
BRAUSI_XDOTOOL=xdotool
```

If your distro uses `chromium-browser`:

```sh
BRAUSI_CHROMIUM=chromium-browser brausi start https://example.com
```

## Orange Pi

See [docs/orangepi.md](docs/orangepi.md).
