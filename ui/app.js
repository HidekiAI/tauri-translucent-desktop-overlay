const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const subtitleBox  = document.getElementById("subtitle-box");
const subtitleText = document.getElementById("subtitle-text");

let cfg = {
  background_opacity: 0.45,
  text_color: "#f5e642",
  font_size_pt: 24,
  min_font_size_pt: 0,
};

// Alternates between two imperceptibly different body background values.
// Changing body's inline style marks the entire body layer dirty, forcing
// WebKit to repaint it — including the area vacated when the pill shrinks.
// Without this, WebKit treats "transparent→transparent" as a no-op and
// leaves old text pixels on the X11 surface (the ghost-text bug).
let _bodyTick = 0;
const BODY_BG = ["rgba(10,10,10,0.02)", "rgba(10,10,10,0.04)"];

function renderText(text) {
  subtitleText.textContent    = text;
  subtitleText.style.color    = cfg.text_color;
  subtitleText.style.fontSize = `${cfg.font_size_pt}pt`;
  subtitleBox.style.background = text
    ? `rgba(10, 10, 10, ${cfg.background_opacity})`
    : "rgba(10, 10, 10, 0.001)"; // near-zero keeps body-layer delta real

  // Force full body-layer repaint in the same frame.
  _bodyTick ^= 1;
  document.body.style.background = BODY_BG[_bodyTick];
}

// --- vertical position cycling (ArrowUp / ArrowDown) ---
const POSITIONS = ["top", "center", "bottom"];
let posIndex = 2;

document.addEventListener("keydown", (e) => {
  if (e.key === "ArrowUp" && posIndex > 0) {
    posIndex--;
    invoke("move_window", { position: POSITIONS[posIndex] }).catch(console.error);
  } else if (e.key === "ArrowDown" && posIndex < POSITIONS.length - 1) {
    posIndex++;
    invoke("move_window", { position: POSITIONS[posIndex] }).catch(console.error);
  }
});

async function init() {
  cfg = await invoke("get_display_config");
  renderText(await invoke("get_hud_text"));

  // Rust emits this after each UDP message.  renderText()'s body-background
  // toggle marks the body layer dirty (instant fix when WebKit is responsive).
  // 150 ms later Rust also sends a synthetic X11 Expose for the full window,
  // forcing a pixel-perfect repaint that clears any ghost from old text.
  await listen("hud-text-changed", (event) => {
    renderText(event.payload);
  });
}

init().catch(console.error);
