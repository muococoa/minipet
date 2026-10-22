import { loadPet, Player } from "./sprite.js";

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const { getCurrentWindow } = window.__TAURI__.window;
const { LogicalSize, PhysicalPosition } = window.__TAURI__.dpi;

// How the pet reacts to you. Times are in seconds.
const BEHAVIOR = {
  activeWithin: 5, // you touched the keyboard or mouse this recently: he works with you
  waitingAfter: 180, // you've been away this long: he waits for you
  dragSettle: 0.25, // he stopped moving this long ago: the drag is over
};

const win = getCurrentWindow();
const canvas = document.getElementById("pet");
const player = new Player(canvas);

const state = {
  base: "idle", // idle, running or waiting, from your activity
  away: false, // you've been gone long enough that he waves when you're back
  drag: null, // "left" or "right" while being dragged
  oneShot: null, // a single animation on top of everything else
  review: false, // shows the review animation until turned off
};

function wanted() {
  if (state.drag) return `running-${state.drag}`;
  if (state.oneShot) return state.oneShot;
  if (state.review) return "review";
  return state.base;
}

function apply(restart = false) {
  const name = wanted();
  if (!restart && player.current?.name === name) return;
  const oneShot = name === state.oneShot;
  player.play(name, () => {
    if (oneShot && state.oneShot === name) {
      state.oneShot = null;
      apply();
    }
  });
}

// Plays once and goes back to what he was doing. Looping ones keep going until you click him.
function playOnce(name) {
  if (!player.has(name)) return;
  state.oneShot = name;
  apply(true);
}

function setReview(on) {
  state.review = on;
  apply();
}

// --- sizing -----------------------------------------------------------------

let movingOurselves = false;

// Resizes the window around the pet, keeping his feet where they are.
async function fitWindow() {
  const { width, height } = player.size;
  const factor = await win.scaleFactor();
  const before = await win.outerSize();
  const pos = await win.outerPosition();
  const newW = Math.round(width * factor);
  const newH = Math.round(height * factor);
  if (before.width === newW && before.height === newH) return;

  movingOurselves = true;
  await win.setSize(new LogicalSize(width, height));
  await win.setPosition(
    new PhysicalPosition(
      pos.x + Math.round((before.width - newW) / 2),
      pos.y + before.height - newH,
    ),
  );
  setTimeout(() => (movingOurselves = false), 200);
}

// Moving to a screen with a different pixel density changes how big a pixel is.
function watchPixelDensity() {
  const query = matchMedia(`(resolution: ${window.devicePixelRatio}dppx)`);
  query.addEventListener(
    "change",
    () => {
      player.layout();
      fitWindow();
      watchPixelDensity();
    },
    { once: true },
  );
}

async function reloadPet() {
  const { pet: name } = await invoke("get_settings");
  const def = await invoke("load_pet");
  player.setPet(await loadPet(name, def));
  await fitWindow();
  apply(true);
}

// --- mouse ------------------------------------------------------------------

let press = null;
let lastX = null;
let settleTimer = null;

canvas.addEventListener("mousedown", (e) => {
  if (e.button === 0) press = { x: e.screenX, y: e.screenY, dragging: false };
});

window.addEventListener("mousemove", (e) => {
  if (!press || press.dragging || !(e.buttons & 1)) return;
  if (Math.hypot(e.screenX - press.x, e.screenY - press.y) < 4) return;
  press.dragging = true;
  state.oneShot = null;
  win.startDragging();
});

window.addEventListener("mouseup", (e) => {
  if (e.button !== 0 || !press) return;
  const wasDrag = press.dragging;
  press = null;
  if (!wasDrag) playOnce("jumping");
});

canvas.addEventListener("contextmenu", (e) => {
  e.preventDefault();
  invoke("show_menu");
});

// The system moves the window during a drag, so the direction comes from here.
win.onMoved(({ payload }) => {
  if (movingOurselves) {
    lastX = payload.x;
    return;
  }
  if (lastX !== null && payload.x !== lastX) {
    const dir = payload.x > lastX ? "right" : "left";
    if (state.drag !== dir) {
      state.drag = dir;
      apply();
    }
  }
  lastX = payload.x;

  clearTimeout(settleTimer);
  settleTimer = setTimeout(() => {
    press = null;
    state.drag = null;
    apply();
    invoke("save_position");
  }, BEHAVIOR.dragSettle * 1000);
});

// --- events from the app ----------------------------------------------------

listen("activity", ({ payload: idleSeconds }) => {
  if (idleSeconds >= BEHAVIOR.waitingAfter) {
    state.away = true;
    state.base = "waiting";
  } else {
    if (state.away) {
      state.away = false;
      playOnce("waving");
    }
    state.base = idleSeconds < BEHAVIOR.activeWithin ? "running" : "idle";
  }
  apply();
});

listen("play", ({ payload: name }) => playOnce(name));

listen("scale", async ({ payload: scale }) => {
  player.setScale(scale);
  await fitWindow();
});

listen("reload", () => reloadPet());

// Real triggers come later. For now these are here for testing from devtools.
window.pet = { playOnce, setReview };

// --- start ------------------------------------------------------------------

async function start() {
  const settings = await invoke("get_settings");
  player.setScale(settings.scale);
  try {
    await reloadPet();
  } catch (err) {
    invoke("log", { message: String(err) });
  }
  await invoke("place_pet");
  await invoke("show_pet");
  watchPixelDensity();
  playOnce("waving");
}

start();
