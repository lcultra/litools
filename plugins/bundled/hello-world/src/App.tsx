import { createSignal, onMount } from "solid-js";
import {
  ready,
  getInfo,
  close,
  setTitle,
  toast,
  storageGet,
  storageSet,
  onEnter,
  onLeave,
} from "@litools/plugin-sdk";

type LogEntry = { message: string; value?: unknown };

export default function App() {
  const [log, setLog] = createSignal<LogEntry>({ message: "Starting..." });

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
    onEnter(() => write("lifecycle.onEnter"));
    onLeave(() => write("lifecycle.onLeave"));
    await run("ready", () => ready());
  });

  return (
    <main>
      <h1>Hello from a litools plugin</h1>
      <p>This page uses @litools/plugin-sdk with Vite + SolidJS.</p>
      <input type="text" placeholder="Try typing here" />
      <div class="actions">
        <button onClick={() => run("getInfo", () => getInfo())}>Get info</button>
        <button
          onClick={() =>
            run("storage round trip", async () => {
              const next = { savedAt: new Date().toISOString() };
              await storageSet("hello-world", next);
              return storageGet("hello-world");
            })
          }
        >
          Storage round trip
        </button>
        <button onClick={() => run("setTitle", () => setTitle("Hello World"))}>
          Set title
        </button>
        <button onClick={() => run("toast", () => toast("Hello from plugin"))}>
          Toast
        </button>
        <button onClick={() => run("close", () => close())}>Close</button>
      </div>
      <pre>
        {log().message}
        {log().value !== undefined ? `\n${JSON.stringify(log().value, null, 2)}` : ""}
      </pre>
    </main>
  );
}
