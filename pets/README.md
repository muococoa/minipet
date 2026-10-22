# Pets

Each pet is a folder. Adding a pet is adding a folder.

```
pets/
  pichi/
    pichi.json      ← same name as the folder
    idle.png
    running-left.png
    ...
```

## The json

```json
{
  "frameWidth": 192,
  "frameHeight": 208,
  "animations": {
    "idle":    { "file": "idle.png",    "frames": 6, "fps": 6,  "loop": true },
    "jumping": { "file": "jumping.png", "frames": 5, "fps": 10, "loop": false, "frameHeight": 272 }
  }
}
```

- Each sheet is one horizontal row of frames, so its width is `frameWidth × frames`.
- `frameWidth` and `frameHeight` at the top apply to every animation. An
  animation can set its own to override them, like `jumping` above.
- Every animation is anchored at the bottom center, so the feet stay put
  whatever the frame size.
- `loop: false` plays once, then the pet goes back to what he was doing.

## Animations the app uses

| Name | When |
| --- | --- |
| `idle` | Default |
| `running` | You're using the computer |
| `waiting` | You've been away a few minutes |
| `waving` | He appears, or you come back |
| `jumping` | You click him |
| `running-left`, `running-right` | You drag him |
| `failed`, `review` | Not triggered yet. Use **Play animation** in the menu |

Any other animation in the json shows up in **Play animation** too.
