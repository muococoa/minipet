// Loads a pet and plays its sprite sheets.
// Every size, frame count and speed comes from the pet's json, so replacing a
// sheet and its line in the json is all it takes to change an animation.

const { invoke } = window.__TAURI__.core;

function petUrl(path) {
  // Windows serves custom protocols over http, macOS and Linux use the scheme directly.
  return navigator.userAgent.includes("Windows")
    ? `http://pet.localhost/${path}`
    : `pet://localhost/${path}`;
}

export async function loadPet(name, def) {
  const bust = `?v=${Date.now()}`;
  const animations = {};

  await Promise.all(
    Object.entries(def.animations).map(async ([key, a]) => {
      const w = a.frameWidth ?? def.frameWidth;
      const h = a.frameHeight ?? def.frameHeight;
      const img = new Image();
      img.src = petUrl(`${name}/${a.file}`) + bust;
      try {
        await img.decode();
      } catch {
        invoke("log", { message: `${name}: could not load ${a.file} for "${key}"` });
        return;
      }
      if (img.naturalWidth !== w * a.frames || img.naturalHeight !== h) {
        invoke("log", {
          message:
            `${name}: ${a.file} is ${img.naturalWidth}×${img.naturalHeight}, ` +
            `but the json says ${a.frames} frames of ${w}×${h} (${w * a.frames}×${h})`,
        });
      }
      animations[key] = { img, w, h, frames: a.frames, fps: a.fps, loop: a.loop };
    }),
  );

  const all = Object.values(animations);
  return {
    name,
    animations,
    maxW: Math.max(...all.map((a) => a.w)),
    maxH: Math.max(...all.map((a) => a.h)),
  };
}

export class Player {
  constructor(canvas) {
    this.canvas = canvas;
    this.ctx = canvas.getContext("2d");
    this.pet = null;
    this.scale = 1;
    this.current = null;
    this.drawnFrame = -1;
    requestAnimationFrame((t) => this.#tick(t));
  }

  // Window size in logical pixels. Scale is screen pixels per sprite pixel.
  get size() {
    const k = this.scale / window.devicePixelRatio;
    return { width: this.pet.maxW * k, height: this.pet.maxH * k };
  }

  setPet(pet) {
    this.pet = pet;
    this.current = null;
    this.layout();
  }

  setScale(scale) {
    this.scale = scale;
    this.layout();
  }

  // The canvas covers the largest frame of any animation, in real screen pixels.
  layout() {
    if (!this.pet) return;
    const { width, height } = this.size;
    this.canvas.width = this.pet.maxW * this.scale;
    this.canvas.height = this.pet.maxH * this.scale;
    this.canvas.style.width = `${width}px`;
    this.canvas.style.height = `${height}px`;
    this.drawnFrame = -1;
  }

  has(name) {
    return Boolean(this.pet?.animations[name]);
  }

  // Starts an animation from its first frame. onEnd runs when a non-looping one finishes.
  play(name, onEnd = null) {
    const anim = this.pet?.animations[name];
    if (!anim) {
      onEnd?.();
      return;
    }
    this.current = { name, anim, start: performance.now(), onEnd, ended: false };
    this.drawnFrame = -1;
  }

  #tick(now) {
    requestAnimationFrame((t) => this.#tick(t));
    const c = this.current;
    if (!c) return;

    let frame = Math.floor(((now - c.start) / 1000) * c.anim.fps);
    if (frame >= c.anim.frames) {
      if (c.anim.loop) {
        frame %= c.anim.frames;
      } else {
        frame = c.anim.frames - 1;
        if (!c.ended) {
          c.ended = true;
          queueMicrotask(() => c.onEnd?.());
        }
      }
    }
    if (frame !== this.drawnFrame) this.#draw(c.anim, frame);
  }

  // Anchored at the bottom center, so the feet stay put whatever the frame height.
  #draw(anim, frame) {
    const { ctx, canvas, scale } = this;
    const w = anim.w * scale;
    const h = anim.h * scale;
    ctx.imageSmoothingEnabled = false;
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.drawImage(
      anim.img,
      frame * anim.w, 0, anim.w, anim.h,
      Math.round((canvas.width - w) / 2), canvas.height - h, w, h,
    );
    this.drawnFrame = frame;
  }
}
