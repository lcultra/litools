import { For, Show } from 'solid-js';
import { startDragging } from '../../bridge/commands';
import type { SearchResult } from '../../bridge/types';
import { Panel } from '../../components/Panel';
import { providerLabel } from './providerLabels';

type PaletteViewProps = {
    error: string | null;
    inputRef: (element: HTMLInputElement) => void;
    loading: boolean;
    onContentElement: (element: HTMLElement) => void;
    onInput: (value: string) => void;
    onKeyDown: (event: KeyboardEvent) => void;
    onResultRun: (result: SearchResult) => void;
    onSubmit: (event: SubmitEvent) => void;
    query: string;
    results: SearchResult[];
    selectedIndex: number;
};

export function PaletteView(props: PaletteViewProps) {
    function handleDragStart(event: PointerEvent) {
        if (event.button !== 0) {
            return;
        }

        void startDragging();
    }

    return (
        <div ref={props.onContentElement} class="p-px">
            <Panel class="grid w-full self-start grid-rows-[auto_auto]" variant="launcher">
                <form onPointerDown={handleDragStart} onSubmit={props.onSubmit}>
                    <input
                        ref={props.inputRef}
                        autofocus
                        class="w-full border-0 border-b border-border bg-transparent px-6 py-6 text-2xl font-medium text-fg outline-none placeholder:text-muted"
                        id="launcher-search"
                        name="launcher-search"
                        on:keydown={props.onKeyDown}
                        onInput={(event) => props.onInput(event.currentTarget.value)}
                        placeholder="搜索应用、命令、文件、插件..."
                        value={props.query}
                    />
                </form>

                <div class="grid max-h-[424px] min-h-0 grid-rows-[1fr] overflow-hidden p-2">
                    <Show when={!props.loading} fallback={<p class="m-0 px-4 py-3 text-sm text-muted">正在搜索...</p>}>
                        <Show when={!props.error} fallback={<p class="m-0 px-4 py-3 text-sm text-danger">{props.error}</p>}>
                            <Show when={props.results.length > 0} fallback={<p class="m-0 px-4 py-3 text-sm text-muted">未找到结果</p>}>
                                <div class="grid auto-rows-[92px] grid-cols-[repeat(auto-fill,minmax(88px,1fr))] gap-2 overflow-y-auto overscroll-contain">
                                    <For each={props.results}>
                                        {(result, index) => (
                                            <button
                                                class="grid cursor-pointer grid-rows-[1fr_auto] place-items-center gap-2 rounded-2xl border border-transparent p-3 text-center text-inherit outline-none transition-colors"
                                                classList={{
                                                    'border-accent/40 bg-accent/15 text-fg': props.selectedIndex === index(),
                                                    'bg-transparent hover:bg-surface-muted/60 focus-visible:bg-surface-muted/60': props.selectedIndex !== index(),
                                                }}
                                                onClick={() => props.onResultRun(result)}
                                                tabindex={-1}
                                                type="button"
                                            >
                                                <span class="grid size-12 place-items-center rounded-2xl bg-app text-sm font-semibold text-muted">
                                                    {providerLabel(result.provider).slice(0, 1)}
                                                </span>
                                                <span class="w-full truncate text-xs font-medium">{result.title}</span>
                                            </button>
                                        )}
                                    </For>
                                </div>
                            </Show>
                        </Show>
                    </Show>
                </div>
            </Panel>
        </div>
    );
}
