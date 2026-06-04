import { createEffect, createSignal, onCleanup, onMount } from 'solid-js';
import { executeResult, hideMainWindow, search } from '../../bridge/commands';
import { onFocusSearch } from '../../bridge/events';
import type { BuiltinCommandEffect, SearchResult } from '../../bridge/types';
import { PaletteView } from './PaletteView';

type PaletteSearchControllerProps = {
    onCommandEffect: (effect: BuiltinCommandEffect) => void;
};

export function PaletteSearchController(props: PaletteSearchControllerProps) {
    let inputElement: HTMLInputElement | undefined;
    let searchRequestId = 0;
    const [query, setQuery] = createSignal('');
    const [results, setResults] = createSignal<SearchResult[]>([]);
    const [selectedIndex, setSelectedIndex] = createSignal(0);
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
            props.onCommandEffect(response.effect);
        } catch (executeError) {
            setError(`执行失败：${String(executeError)}`);
        }
    }

    function handleSubmit(event: SubmitEvent) {
        event.preventDefault();
        void runResult(results()[selectedIndex()]);
    }

    function selectResultByOffset(offset: number) {
        setSelectedIndex((current) => (results().length ? (current + offset + results().length) % results().length : 0));
        queueMicrotask(() => inputElement?.focus());
    }

    function handleKeyDown(event: KeyboardEvent) {
        if (event.key === 'Tab') {
            event.preventDefault();
            event.stopPropagation();
            selectResultByOffset(event.shiftKey ? -1 : 1);
            return;
        }

        if (event.key === 'ArrowDown') {
            event.preventDefault();
            selectResultByOffset(1);
            return;
        }

        if (event.key === 'ArrowUp') {
            event.preventDefault();
            selectResultByOffset(-1);
            return;
        }

        if (event.key === 'Escape') {
            event.preventDefault();
            setQuery('');
            setError(null);
            void hideMainWindow();
        }
    }

    return (
        <PaletteView
            error={error()}
            inputRef={(element) => {
                inputElement = element;
            }}
            loading={loading()}
            onInput={setQuery}
            onKeyDown={handleKeyDown}
            onResultRun={(result) => void runResult(result)}
            onSubmit={handleSubmit}
            query={query()}
            results={results()}
            selectedIndex={selectedIndex()}
        />
    );
}
