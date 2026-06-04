import { For, Show, createEffect, createSignal, onCleanup, onMount } from 'solid-js';
import { executeResult, hideMainWindow, search } from '../../bridge/commands';
import { onFocusSearch } from '../../bridge/events';
import type { BuiltinCommandEffect, SearchResult } from '../../bridge/types';

type CommandPaletteProps = {
  onCommandEffect: (effect: BuiltinCommandEffect) => void;
};

export function CommandPalette(props: CommandPaletteProps) {
  let inputElement: HTMLInputElement | undefined;
  let searchRequestId = 0;
  const [query, setQuery] = createSignal('');
  const [results, setResults] = createSignal<SearchResult[]>([]);
  const [selectedIndex, setSelectedIndex] = createSignal(0);
  const [message, setMessage] = createSignal<string | null>(null);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

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
    const requestId = ++searchRequestId;
    setMessage(null);
    setError(null);
    setLoading(true);

    const timeout = window.setTimeout(() => {
      search(currentQuery)
        .then((nextResults) => {
          if (requestId !== searchRequestId) {
            return;
          }

          setResults(nextResults);
          setSelectedIndex(0);
        })
        .catch((searchError) => {
          if (requestId === searchRequestId) {
            setError(`搜索失败：${String(searchError)}`);
            setResults([]);
          }
        })
        .finally(() => {
          if (requestId === searchRequestId) {
            setLoading(false);
          }
        });
    }, 120);

    onCleanup(() => window.clearTimeout(timeout));
  });

  async function runResult(result: SearchResult | undefined) {
    const [firstAction] = result?.actions ?? [];

    if (!result || !firstAction) {
      return;
    }

    try {
      const response = await executeResult(result.id, firstAction.id);
      setMessage(response.message);
      props.onCommandEffect(response.effect);
    } catch (executeError) {
      setError(`执行失败：${String(executeError)}`);
    }
  }

  function handleSubmit(event: SubmitEvent) {
    event.preventDefault();
    void runResult(results()[selectedIndex()]);
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'ArrowDown') {
      event.preventDefault();
      setSelectedIndex((current) => (results().length ? (current + 1) % results().length : 0));
      return;
    }

    if (event.key === 'ArrowUp') {
      event.preventDefault();
      setSelectedIndex((current) => (results().length ? (current - 1 + results().length) % results().length : 0));
      return;
    }

    if (event.key === 'Escape') {
      event.preventDefault();
      setQuery('');
      setMessage(null);
      setError(null);
      void hideMainWindow();
    }
  }

  return (
    <section class="w-[min(720px,calc(100vw-32px))] overflow-hidden rounded-[20px] border border-border bg-surface shadow-[var(--shadow-panel)] backdrop-blur">
      <form onSubmit={handleSubmit}>
        <input
          ref={inputElement}
          autofocus
          class="w-full border-0 border-b border-border bg-transparent px-6 py-[22px] text-xl text-inherit outline-none placeholder:text-muted"
          onInput={(event) => setQuery(event.currentTarget.value)}
          onKeyDown={handleKeyDown}
          placeholder="搜索应用、命令、文件、插件..."
          value={query()}
        />
      </form>
      <div class="grid p-2">
        <Show when={!loading()} fallback={<p class="m-0 px-4 py-3 text-sm text-muted">正在搜索...</p>}>
          <Show when={!error()} fallback={<p class="m-0 px-4 py-3 text-sm text-danger">{error()}</p>}>
            <Show when={results().length > 0} fallback={<p class="m-0 px-4 py-3 text-sm text-muted">未找到结果</p>}>
              <For each={results()}>
                {(result, index) => (
                  <button
                    class="grid cursor-pointer gap-1 rounded-xl border-0 px-4 py-3 text-left text-inherit transition-colors hover:bg-surface-muted"
                    classList={{ 'bg-surface-muted': selectedIndex() === index(), 'bg-transparent': selectedIndex() !== index() }}
                    onClick={() => void runResult(result)}
                    onMouseEnter={() => setSelectedIndex(index())}
                    type="button"
                  >
                    <span class="font-semibold">{result.title}</span>
                    <div class="flex items-center justify-between gap-4 text-sm text-muted">
                      <span>{result.subtitle}</span>
                      <span>{providerLabel(result.provider)} · {result.actions[0]?.label ?? '执行'}</span>
                    </div>
                  </button>
                )}
              </For>
            </Show>
          </Show>
        </Show>
      </div>
      <Show when={message()}>
        <p class="m-0 px-6 pb-[18px] text-sm text-muted">{message()}</p>
      </Show>
    </section>
  );
}

function providerLabel(provider: string) {
  if (provider === 'commands') {
    return '命令';
  }

  return provider;
}
