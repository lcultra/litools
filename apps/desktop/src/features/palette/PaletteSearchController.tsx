import { createEffect, createSignal, onCleanup, onMount } from 'solid-js';
import { executeResult, hideMainWindow, resizeMainWindowHeight, search } from '../../bridge/commands';
import { onFocusSearch } from '../../bridge/events';
import type { BuiltinCommandEffect, SearchResult } from '../../bridge/types';
import { PaletteView } from './PaletteView';

const MIN_LAUNCHER_HEIGHT = 96;
const MAX_LAUNCHER_HEIGHT = 520;

type PaletteSearchControllerProps = {
    onCommandEffect: (effect: BuiltinCommandEffect) => void;
};

export function PaletteSearchController(props: PaletteSearchControllerProps) {
    let inputElement: HTMLInputElement | undefined;
    let resizeFrame = 0;
    let lastSyncedHeight = 0;
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

    function clampLauncherHeight(height: number) {
        return Math.min(Math.max(Math.ceil(height), MIN_LAUNCHER_HEIGHT), MAX_LAUNCHER_HEIGHT);
    }

    function scheduleWindowHeightSync(height: number) {
        const nextHeight = clampLauncherHeight(height);

        if (nextHeight === lastSyncedHeight) {
            return;
        }

        window.cancelAnimationFrame(resizeFrame);
        resizeFrame = window.requestAnimationFrame(() => {
            lastSyncedHeight = nextHeight;
            void resizeMainWindowHeight(nextHeight);
        });
    }

    function measureContentHeight(element: HTMLElement) {
        return element.getBoundingClientRect().height;
    }

    function handleContentElement(element: HTMLElement) {
        const observer = new ResizeObserver(() => scheduleWindowHeightSync(measureContentHeight(element)));
        observer.observe(element);
        scheduleWindowHeightSync(measureContentHeight(element));

        onCleanup(() => {
            window.cancelAnimationFrame(resizeFrame);
            observer.disconnect();
        });
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
            onContentElement={handleContentElement}
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
