import { createSignal, Show } from "solid-js";
import logo from "./assets/logo.svg";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";

function App() {
  const [greetMsg, setGreetMsg] = createSignal("");
  const [name, setName] = createSignal("");
  const [showDialog, setShowDialog] = createSignal(false);
  const [currentUrl, setCurrentUrl] = createSignal("");
  const [customUrl, setCustomUrl] = createSignal("");

  async function greet() {
    const msg = await invoke<string>("greet", { name: name() });
    setGreetMsg(msg);
    setShowDialog(true);
  }

  async function loadUrl() {
    const url = await invoke<string>("get_url");
    setCurrentUrl(url);
  }

  async function sendCustomUrl() {
    if (customUrl()) {
      const url = await invoke<string>("send_custom_url", { url: customUrl() });
      setCurrentUrl(url);
    }
  }

  return (
    <div class="container">
      <h1>Welcome to Tauri!</h1>

      <div class="row">
        <a href="https://vitejs.dev" target="_blank">
          <img src="/vite.svg" class="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" class="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://solidjs.com" target="_blank">
          <img src={logo} class="logo solid" alt="Solid logo" />
        </a>
      </div>

      <p>Click on the Tauri, Vite, and Solid logos to learn more.</p>

      <div class="url-section" style={{ margin: "2rem 0", padding: "1rem", border: "1px solid #ccc", "border-radius": "8px" }}>
        <h2>Web Content Renderer</h2>
        
        <div class="controls" style={{ margin: "1rem 0", display: "flex", gap: "1rem", "align-items": "center", "flex-wrap": "wrap" }}>
          <button onClick={loadUrl}>Load YouTube Login</button>
          
          <form
            style={{ display: "flex", gap: "0.5rem", "align-items": "center" }}
            onSubmit={(e) => {
              e.preventDefault();
              sendCustomUrl();
            }}
          >
            <input
              onChange={(e) => setCustomUrl(e.currentTarget.value)}
              placeholder="Enter URL (e.g., github.com)..."
              value={customUrl()}
              style={{ "min-width": "200px", padding: "0.5rem" }}
            />
            <button type="submit">Load URL</button>
          </form>
        </div>

        <Show when={currentUrl()}>
          <div style={{ margin: "1rem 0" }}>
            <div style={{
              display: "flex",
              "justify-content": "space-between",
              "align-items": "center",
              padding: "0.5rem",
              background: "#f0f0f0",
              "border-radius": "4px 4px 0 0",
              "font-size": "0.9rem"
            }}>
              <span>Rendering: {currentUrl()}</span>
              <button
                onClick={() => setCurrentUrl("")}
                style={{
                  background: "#ff4444",
                  color: "white",
                  border: "none",
                  padding: "0.25rem 0.5rem",
                  "border-radius": "3px",
                  cursor: "pointer"
                }}
              >
                Close
              </button>
            </div>
            <iframe
              src={currentUrl()}
              style={{
                width: "100%",
                height: "600px",
                border: "1px solid #ccc",
                "border-radius": "0 0 4px 4px"
              }}
              title="Web Content"
            />
          </div>
        </Show>
      </div>

      <form
        class="row"
        onSubmit={(e) => {
          e.preventDefault();
          greet();
        }}
      >
        <input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button type="submit">Greet</button>
      </form>

      <Show when={showDialog()}>
        <div
          style={{
            position: "fixed",
            top: 0,
            left: 0,
            width: "100vw",
            height: "100vh",
            background: "rgba(0,0,0,0.4)",
            display: "flex",
            "align-items": "center",
            "justify-content": "center",
            "z-index": 1000,
          }}
        >
          <div
            style={{
              background: "white",
              padding: "2rem",
              "border-radius": "8px",
              "min-width": "300px",
              "text-align": "center",
            }}
          >
            <p>{greetMsg()}</p>
            <button onClick={() => setShowDialog(false)}>Close</button>
          </div>
        </div>
      </Show>
    </div>
  );
}

export default App;
