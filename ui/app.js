// Tauri injects window.__TAURI__ when app.withGlobalTauri = true in tauri.conf.json
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const subtitleBox  = document.getElementById("subtitle-box");
const subtitleText = document.getElementById("subtitle-text");

let cfg = {
  background_opacity: 0.72,
  text_color: "#f5e642",
  font_size_pt: 24,
  min_font_size_pt: 0,
};

function applyText(text) {
  // Erase phase: keep old text but paint fully opaque for 3 frames.
  // Alpha=1 forces the X11 compositor to REPLACE old content (not blend),
  // clearing accumulated ghost pixels. Then show new text at normal opacity.
  subtitleBox.style.background = "rgba(10, 10, 10, 1.0)";
  subtitleText.style.color = "rgba(0,0,0,0)"; // text invisible during erase
  requestAnimationFrame(() => {
    subtitleBox.style.background = "rgba(10, 10, 10, 1.0)";
    requestAnimationFrame(() => {
      subtitleBox.style.background = "rgba(10, 10, 10, 1.0)";
      requestAnimationFrame(() => renderText(text));
    });
  });
}

function renderText(text) {
  subtitleText.textContent = text;
  subtitleBox.style.background = `rgba(10, 10, 10, ${cfg.background_opacity})`;
  subtitleText.style.color = cfg.text_color;

  const usesShrink =
    cfg.min_font_size_pt > 0 && cfg.min_font_size_pt < cfg.font_size_pt;

  if (usesShrink) {
    subtitleText.style.fontSize   = `${cfg.font_size_pt}pt`;
    subtitleText.style.whiteSpace = "nowrap";
    requestAnimationFrame(() => {
      if (subtitleText.scrollWidth > subtitleText.clientWidth) {
        subtitleText.style.fontSize = `${cfg.min_font_size_pt}pt`;
      }
      subtitleText.style.whiteSpace = "pre-wrap";
    });
  } else {
    subtitleText.style.fontSize   = `${cfg.font_size_pt}pt`;
    subtitleText.style.whiteSpace = "pre-wrap";
  }
}

async function init() {
  cfg = await invoke("get_display_config");

  applyText(await invoke("get_hud_text"));

  await listen("hud-text-changed", (event) => {
    applyText(event.payload);
  });
}

init().catch(console.error);
