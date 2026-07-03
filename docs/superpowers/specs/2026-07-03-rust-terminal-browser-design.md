# Rust Terminal Browser Design

## Goal

Build a lightweight terminal utility in Rust that controls a real Chromium instance. The first version should replace the current Bash proof of concept with a single binary that starts Chromium in a virtual X11 display, renders the browser framebuffer into the terminal, and sends mouse/keyboard/navigation events to Chromium.

The project is explicitly not a browser engine. It will not parse or render HTML, CSS, or JavaScript itself. Chromium remains the web engine for real-world site compatibility, while Rust provides a small, predictable control layer.

## Target Platform

Primary target is the Orange Pi PC armv7 32-bit Linux environment where the Bash prototype already worked.

The runtime model should stay friendly to low-power hardware:

- avoid Electron and heavyweight UI stacks;
- use one Rust process plus small Linux tools;
- keep Chromium flags biased toward software rendering and headless-display stability;
- make external commands configurable because package names differ across distros.

## Runtime Dependencies

The first implementation may shell out to existing tools:

- `Xvfb` for virtual display;
- `chromium` or `chromium-browser` for the web engine;
- `scrot` for screenshot capture;
- `chafa` for terminal rendering;
- `xdotool` for X11 input events.

Rust owns process orchestration, command parsing, state paths, cleanup, and error reporting. Later versions can replace individual shell-outs with native X11 or image code if needed, but the core product remains a light browser-control utility.

## Command Interface

The first binary should expose a CLI named `brausi`:

```text
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
brausi stop
brausi status
```

Coordinates are framebuffer pixels, not terminal row/column cells. With the default display this means `0..1023` for `x` and `0..767` for `y`.

## Process Model

`brausi start [url]` should:

1. stop stale processes from a previous Brausi session when they are clearly owned by the configured display;
2. start `Xvfb` on display `:99` with a default `1024x768x24` screen;
3. start Chromium using the same display;
4. store process IDs and runtime metadata under a state directory;
5. return once Chromium has had enough time to open the initial page.

The state directory defaults to `~/.brausi`, with runtime files under `~/.brausi/run` and browser profile data under `~/.brausi/profile`.

`brausi stop` should terminate the tracked Chromium and Xvfb processes and remove transient files. It should avoid broad `killall` behavior by default.

## Rendering

`brausi view` should run a continuous render loop:

1. capture the active Chromium window or virtual display to a PNG;
2. clear/redraw the terminal;
3. render the image with `chafa --symbols=space`;
4. repeat at a configurable interval, defaulting to 200 ms.

The viewport reserves no terminal UI chrome in the first implementation. Address bar and buttons are exposed as commands first. A later interactive mode can render a top bar and translate terminal mouse coordinates into framebuffer pixels.

## Input and Navigation

Input commands use `xdotool` on the configured display:

- `click x y`: `mousemove x y` followed by left click;
- `move x y`: pointer movement only;
- `down x y`: pointer movement plus mouse down;
- `up x y`: pointer movement plus mouse up;
- `type text`: send text to the focused element;
- `go url`: focus Chromium address bar, type URL, press Return;
- `back`: browser back shortcut;
- `forward`: browser forward shortcut;
- `reload`: browser reload shortcut.

The implementation should set `DISPLAY` only for child processes that need it, not globally for the parent shell.

## Configuration

The first version should support constants or simple environment variables for:

- display number, default `:99`;
- screen width, default `1024`;
- screen height, default `768`;
- color depth, default `24`;
- capture interval, default `200ms`;
- Chromium command path;
- Xvfb, scrot, chafa, and xdotool command paths.

A config file is out of scope for the first pass.

## Error Handling

Commands should fail with direct messages when dependencies are missing, the browser is not running, coordinates are invalid, or child processes fail to start.

`status` should report whether tracked Xvfb and Chromium processes appear alive and where runtime files are stored.

## Testing and Verification

The first implementation should include unit tests for argument parsing, coordinate validation, and command construction where possible.

Manual verification on the target board should cover:

- start Chromium with a URL;
- view page pixels in the terminal;
- click a visible coordinate;
- type text into a focused input;
- navigate with `go`, `back`, `forward`, and `reload`;
- stop without leaving tracked processes behind.

## Future Work

After the command-based Rust v1 works, add `brausi tui`:

- terminal top bar with back, forward, reload, and address input;
- keyboard shortcuts;
- optional terminal mouse capture;
- terminal-cell to framebuffer-coordinate conversion;
- cleaner live redraw without full-screen flicker where terminal support allows it.
