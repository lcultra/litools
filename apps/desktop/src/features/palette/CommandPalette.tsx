import { For, Show, createEffect, createSignal, onCleanup, onMount } from 'solid-js';
import { executeResult, hideMainWindow, search } from '../../bridge/commands';
import { onFocusSearch } from '../../bridge/events';
import type { BuiltinCommandEffect, SearchResult } from '../../bridge/types';

type CommandPaletteProps = {
  onCommandEffect: (effect: BuiltinCommandEffect) => void;
};

export function CommandPalette(props: CommandPaletteProps) {
  let inputElement: HTMLInputElement | undefined;
  const [query, setQuery] = createSignal('');
  const [results, setResults] = createSignal<SearchResult[]>([]);
  const [selectedIndex, setSelectedIndex] = createSignal(0);
  const [message, setMessage] = createSignal<string | null>(null);

  onMount(() => {
    inputElement?.focus();
    const unsubscribe = onFocusSearch(() => {
      inputElement?.focus();
      inputElement?.select();
    });

    onCleanup(() => {
      void unsubscribe.then((dispose) => dispose());
    });
  });

  createEffect(() => {
    const currentQuery = query();

    search(currentQuery)
      .then((nextResults) => {
        setResults(nextResults);
        setSelectedIndex(0);
      })
      .catch((error) => setMessage(String(error)));
  });

  async function runResult(result: SearchResult | undefined) {
    const [firstAction] = result?.actions ?? [];

    if (!result || !firstAction) {
      return;
    }

    const response = await executeResult(result.id, firstAction.id);
    setMessage(response.message);
    props.onCommandEffect(response.effect);
  }

  function handleSubmit(event: SubmitEvent) {
    event.preventDefault();
    void runResult(results()[selectedIndex()]);
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'ArrowDown') {
      event.preventDefault();
      setSelectedIndex((current) => Math.min(current + 1, Math.max(results().length - 1, 0)));
      return;
    }

    if (event.key === 'ArrowUp') {
      event.preventDefault();
      setSelectedIndex((current) => Math.max(current - 1, 0));
      return;
    }

    if (event.key === 'Escape') {
      event.preventDefault();
      setQuery('');
      setMessage(null);
      void hideMainWindow();
    }
  }

  return (
    <section class="w-[min(720px,calc(100vw-32px))] overflow-hidden rounded-[20px] border border-white/10 bg-[#1b1e26]/95 shadow-[0_24px_80px_rgba(0,0,0,0.45)]">
      <form onSubmit={handleSubmit}>
        <input
          ref={inputElement}
          autofocus
          class="w-full border-0 border-b border-white/10 bg-transparent px-6 py-[22px] text-xl text-inherit outline-none"
          onInput={(event) => setQuery(event.currentTarget.value)}
          onKeyDown={handleKeyDown}
          placeholder="Search apps, commands, files, plugins..."
          value={query()}
        />
      </form>
      <div class="grid p-2">
        <For each={results()}>
          {(result, index) => (
            <button
              class="grid cursor-pointer gap-1 rounded-xl border-0 px-4 py-3 text-left text-inherit transition-colors hover:bg-white/10"
              classList={{ 'bg-white/10': selectedIndex() === index(), 'bg-transparent': selectedIndex() !== index() }}
              onClick={() => void runResult(result)}
              onMouseEnter={() => setSelectedIndex(index())}
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
