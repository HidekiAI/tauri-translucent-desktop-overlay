import { createEffect, createSignal, onCleanup, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

interface DisplayConfig {
  background_opacity: number;
  text_color: string;
  font_size_pt: number;
  min_font_size_pt: number;
}

const defaults: DisplayConfig = {
  background_opacity: 0.72,
  text_color: "#f5e642",
  font_size_pt: 24,
  min_font_size_pt: 0,
};

export default function App() {
  const [cfg, setCfg] = createSignal<DisplayConfig>(defaults);
  const [text, setText] = createSignal("");
  const [activeFontSize, setActiveFontSize] = createSignal(defaults.font_size_pt);
  const [wrap, setWrap] = createSignal(true);
  let textRef: HTMLParagraphElement | undefined;
  let containerRef: HTMLDivElement | undefined;

  const resizeWindow = () => {
    if (containerRef) {
      invoke("set_window_height", { height: containerRef.offsetHeight });
    }
  };

  onMount(async () => {
    const [loadedCfg, initialText] = await Promise.all([
      invoke<DisplayConfig>("get_display_config"),
      invoke<string>("get_hud_text"),
    ]);
    setCfg(loadedCfg);
    setText(initialText);

    const unlisten = await listen<string>("hud-text-changed", (event) => {
      setText(event.payload);
    });
    onCleanup(unlisten);
  });

  // Recalculate font size and resize window whenever text or config changes
  createEffect(() => {
    const c = cfg();
    text(); // track as dependency

    const minSize = c.min_font_size_pt;
    const usesShrink = minSize > 0 && minSize < c.font_size_pt;

    if (usesShrink) {
      // Render at full size with nowrap to measure whether it overflows
      setActiveFontSize(c.font_size_pt);
      setWrap(false);

      setTimeout(() => {
        if (textRef && textRef.scrollWidth > textRef.clientWidth) {
          setActiveFontSize(minSize);
        }
        setWrap(true);
        // Resize after wrap/font are settled
        setTimeout(resizeWindow, 0);
      }, 0);
    } else {
      setActiveFontSize(c.font_size_pt);
      setWrap(true);
      setTimeout(resizeWindow, 0);
    }
  });

  return (
    <div ref={containerRef} class="hud-root">
      <div
        class="subtitle-box"
        style={{ background: `rgba(10, 10, 10, ${cfg().background_opacity})` }}
      >
        <p
          ref={textRef}
          class="subtitle-text"
          style={{
            color: cfg().text_color,
            "font-size": `${activeFontSize()}pt`,
            "white-space": wrap() ? "normal" : "nowrap",
          }}
        >
          {text()}
        </p>
      </div>
    </div>
  );
}
