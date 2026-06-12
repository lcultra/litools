import { onCleanup, onMount } from 'solid-js';
import { getCurrentSurfaceMetadata } from '../bridge/commands';
import { onSurfaceMetadataChanged } from '../bridge/events';
import { setHostWindowLabel } from '../shared/store';

export function useAppEvents() {
    onMount(() => {
        void getCurrentSurfaceMetadata().then((m) => setHostWindowLabel(m?.hostWindowLabel ?? 'main'));

        const unsubMeta = onSurfaceMetadataChanged((m) => setHostWindowLabel(m.hostWindowLabel));

        onCleanup(() => {
            void unsubMeta.then((d) => d());
        });
    });
}
