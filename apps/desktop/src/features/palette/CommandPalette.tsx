import { For, Show, createEffect, createSignal } from 'solid-js';
import { executeResult, search } from '../../bridge/commands';
import type { SearchResult } from '../../bridge/types';

export function CommandPalette() {
  const [query, setQuery] = createSignal('');
  const [results, setResults] = createSignal<SearchResult[]>([]);
  const [message, setMessage] = createSignal<string | null>(null);

  createEffect(() => {
    const currentQuery = query();

    search(currentQuery)
      .then(setResults)
      .catch((error) => setMessage(String(error)));
  });

  async function handleSubmit(event: SubmitEvent) {
    event.preventDefault();
    const [firstResult] = results();
    const [firstAction] = firstResult?.actions ?? [];

    if (!firstResult || !firstAction) {
      return;
    }

    const response = await executeResult(firstResult.id, firstAction.id);
    setMessage(response);
  }

  return (
    <section class="w-[min(720px,calc(100vw-32px))] overflow-hidden rounded-[20px] border border-white/10 bg-[#1b1e26]/95 shadow-[0_24px_80px_rgba(0,0,0,0.45)]">
      <form onSubmit={handleSubmit}>
        <input
          autofocus
          class="w-full border-0 border-b border-white/10 bg-transparent px-6 py-[22px] text-xl text-inherit outline-none"
          onInput={(event) => setQuery(event.currentTarget.value)}
          placeholder="Search apps, commands, files, plugins..."
          value={query()}
        />
      </form>
      <div class="grid p-2">
        <For each={results()}>
          {(result) => (
            <button
              class="grid cursor-pointer gap-1 rounded-xl border-0 bg-transparent px-4 py-3 text-left text-inherit hover:bg-white/10"
              type="button"
            >
              <span class="font-semibold">{result.title}</span>
              <Show when={result.subtitle}>
                <span class="text-sm text-white/60">{result.subtitle}</span>
              </Show>
            </button>
          )}
        </For>
      </div>
      <Show when={message()}>
        <p class="m-0 px-6 pb-[18px] text-sm text-white/60">{message()}</p>
      </Show>
    </section>
  );
}
