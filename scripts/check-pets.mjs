// Checks every pet folder: each sprite sheet's size must match its line in the json.
// Run with: npm run check-pets

import { readFileSync, readdirSync, existsSync } from "node:fs";
import { join } from "node:path";

const petsDir = new URL("../pets/", import.meta.url).pathname;
let problems = 0;

function pngSize(path) {
  const bytes = readFileSync(path);
  return { width: bytes.readUInt32BE(16), height: bytes.readUInt32BE(20) };
}

for (const name of readdirSync(petsDir)) {
  const jsonPath = join(petsDir, name, `${name}.json`);
  if (!existsSync(jsonPath)) continue;
  const def = JSON.parse(readFileSync(jsonPath, "utf8"));
  console.log(`\n${name}`);

  for (const [key, a] of Object.entries(def.animations)) {
    const w = a.frameWidth ?? def.frameWidth;
    const h = a.frameHeight ?? def.frameHeight;
    const file = join(petsDir, name, a.file);
    if (!existsSync(file)) {
      console.log(`  ✗ ${key}: ${a.file} is missing`);
      problems++;
      continue;
    }
    const size = pngSize(file);
    const expected = { width: w * a.frames, height: h };
    const ok = size.width === expected.width && size.height === expected.height;
    if (!ok) problems++;
    console.log(
      `  ${ok ? "✓" : "✗"} ${key}: ${a.frames} frames of ${w}×${h} at ${a.fps} fps` +
        (ok ? "" : `, but ${a.file} is ${size.width}×${size.height} (expected ${expected.width}×${expected.height})`),
    );
  }
}

console.log(problems ? `\n${problems} problem(s) found.` : "\nAll good.");
process.exit(problems ? 1 : 0);
