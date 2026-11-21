# Scripts

One-off and maintenance scripts that aren't part of the app build. Run them
from the repo root. Everyday commands (dev, test, lint, build) are pnpm scripts
in the root `package.json`, not here.

| Script | What it does |
| --- | --- |
| [screenshots/render.sh](screenshots/render.sh) | Renders each screen in `design/canvas/` to `docs/screenshots/<name>.jpg` with headless Chrome. Run it after updating `design/`. |

```sh
sh scripts/screenshots/render.sh
```

`screenshots/render.sh` needs Google Chrome. Set `CHROME` to use another
Chromium build. `screenshots/support.js` stands in for the canvas runtime: it
renders each screen once, in its default state.

## Adding a script

Give each script its own folder (`scripts/<name>/`) when it needs helper
files, and add a row to the table above.
