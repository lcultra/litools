import { createSignal, For, Show } from 'solid-js';
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

function ResultIcon(props: { result: SearchResult }) {
    const [failed, setFailed] = createSignal(false);
    const fallback = () => providerLabel(props.result.provider).slice(0, 1);

    return (
        <span class="grid size-10 place-items-center overflow-hidden text-sm font-semibold text-muted">
            <Show when={props.result.iconUri && !failed()} fallback={fallback()}>
                <img alt="" class="size-10 object-contain" draggable={false} onError={() => setFailed(true)} src={props.result.iconUri ?? undefined} />
            </Show>
        </span>
    );
}

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
                                <div class="grid auto-rows-[82px] grid-cols-9 gap-2 overflow-y-auto overscroll-contain">
                                    <For each={props.results}>
                                        {(result, index) => (
                                            <button
                                                class="grid cursor-pointer grid-rows-[1fr_auto] place-items-center gap-1.5 rounded-2xl border border-transparent p-2 text-center text-inherit outline-none transition-colors"
                                                classList={{
                                                    'border-accent/40 bg-accent/15 text-fg': props.selectedIndex === index(),
                                                    'bg-transparent hover:bg-surface-muted/60 focus-visible:bg-surface-muted/60': props.selectedIndex !== index(),
                                                }}
                                                onClick={() => props.onResultRun(result)}
                                                tabindex={-1}
                                                type="button"
                                            >
                                                <ResultIcon result={result} />
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
