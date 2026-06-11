import { createSignal, onMount } from "solid-js";

type LogEntry = { message: string; value?: unknown };

export default function App() {
  const [log, setLog] = createSignal<LogEntry>({ message: "Waiting for window.litools..." });

  function write(message: string, value?: unknown) {
    setLog({ message, value });
  }

  async function run(label: string, action: () => Promise<unknown>) {
    try {
      const value = await action();
      write(label, value);
    } catch (error) {
      write(`${label} failed`, error);
    }
  }

  onMount(async () => {
    if (!window.litools) {
      write("window.litools is not available");
      return;
    }

    window.litools.lifecycle.onEnter(() => write("lifecycle.onEnter"));
    window.litools.lifecycle.onLeave(() => write("lifecycle.onLeave"));
    await run("runtime.ready", () => window.litools.runtime.ready());
  });

  return (
    <main>
      <h1>Hello from a litools plugin</h1>
      <p>This page is loaded through the plugin runtime with Vite + SolidJS.</p>
      <input type="text" placeholder="Try typing here" />
      <div class="actions">
        <button onClick={() => run("runtime.getInfo", () => window.litools.runtime.getInfo())}>
          Get info
        </button>
        <button
          onClick={() =>
            run("storage round trip", async () => {
              const next = { savedAt: new Date().toISOString() };
              await window.litools.storage.set("hello-world", next);
              return window.litools.storage.get("hello-world");
            })
          }
        >
          Storage round trip
        </button>
        <button onClick={() => run("ui.setTitle", () => window.litools.ui.setTitle("Hello World"))}>
          Set title
        </button>
        <button onClick={() => run("ui.toast", () => window.litools.ui.toast("Hello from plugin"))}>
          Toast
        </button>
        <button onClick={() => run("ui.close", () => window.litools.ui.close())}>
          Close
        </button>
      </div>
      <pre>{log().message}{log().value !== undefined ? `\n${JSON.stringify(log().value, null, 2)}` : ""}</pre>
    </main>
  );
}
