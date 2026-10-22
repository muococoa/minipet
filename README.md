# Minipet

A free, open source desk pet. A small pixel-art pet floats above all your apps.
Mac first, Windows later.

## Run it

You need Xcode's command line tools, Rust and Node.

```bash
xcode-select --install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Install Node (LTS) from https://nodejs.org if `node -v` says it's missing.

Then, in this folder:

```bash
npm install
npm run dev
```

The first run takes a few minutes while Rust compiles. Later runs are quick.

## Using it

- Drag the pet to move him. He remembers where you left him.
- Click him to make him jump.
- Right-click him, or click his icon in the menu bar, for size, pets, test
  animations, reload and quit.

## Changing animations

Everything about a pet comes from its folder in `pets/`. To redo an animation,
replace its sprite sheet, update its line in the json, then choose
**Reload pet files** from the menu. No code changes, no restart.

To check that every sheet matches its json:

```bash
npm run check-pets
```

See [pets/README.md](pets/README.md) for the format.

## How it works

- `src-tauri/` is the native app (Rust, Tauri 2).
  - `panel.rs` makes the window a macOS panel so it floats over full-screen apps.
  - `activity.rs` reads the macOS idle timer. No Accessibility or Input
    Monitoring permission needed.
  - `pets.rs` serves the `pets/` folder to the page.
- `ui/` is the pet itself.
  - `sprite.js` loads a pet and plays its sheets.
  - `main.js` decides which animation to play, and when.
