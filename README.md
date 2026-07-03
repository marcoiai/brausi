# Brausi

Brausi is a lightweight terminal browser controller implemented as a Bash
script. It does not implement a browser engine. It starts Chromium in `Xvfb`,
renders the virtual display into a terminal with `scrot`/`chafa`, and sends
input through `xdotool`.

## Commands

```sh
./bin/brausi start [url]
./bin/brausi view
./bin/brausi click <x> <y>
./bin/brausi move <x> <y>
./bin/brausi down <x> <y>
./bin/brausi up <x> <y>
./bin/brausi type <text>
./bin/brausi go <url>
./bin/brausi back
./bin/brausi forward
./bin/brausi reload
./bin/brausi status
./bin/brausi stop
```

Coordinates are framebuffer pixels. With the default display size, valid values
are `x=0..1023` and `y=0..767`.

## Install

```sh
sudo install -m 0755 bin/brausi /usr/local/bin/brausi
```

There is no build step for the primary implementation.

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
BRAUSI_VIEW_INTERVAL=0.2
```

`BRAUSI_RENDER_COLS` and `BRAUSI_RENDER_LINES` are optional overrides. When
unset, `view` uses `tput cols` and `tput lines`, like the original shell loop.

If your distro uses `chromium-browser`:

```sh
BRAUSI_CHROMIUM=chromium-browser ./bin/brausi start https://example.com
```

If terminal output wraps or looks skewed, force a smaller render area:

```sh
BRAUSI_RENDER_COLS=79 BRAUSI_RENDER_LINES=23 ./bin/brausi view
```

## Rust Prototype

The repository still contains the initial Rust prototype for reference, but the
recommended command is the Bash script in `bin/brausi`.

## Orange Pi

See [docs/orangepi.md](docs/orangepi.md).
