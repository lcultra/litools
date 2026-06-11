import { createSignal, Show } from 'solid-js';
import type { SearchResult } from '../bridge/types';
import { providerLabel } from '../shared/strings';

export function ResultIcon(props: { result: SearchResult }) {
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
