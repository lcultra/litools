import { onCleanup, onMount } from 'solid-js';
import { getCurrentSurfaceMetadata } from '../bridge/commands';
import { onSurfaceMetadataChanged } from '../bridge/events';
import { baseInfo, initBaseInfo, setHostWindowLabel } from '../shared/store';

export function useAppEvents() {
    onMount(() => {
        initBaseInfo();

        void getCurrentSurfaceMetadata().then((m) => {
            const fallback = baseInfo()?.mainWindowLabel ?? '';
            setHostWindowLabel(m?.hostWindowLabel ?? fallback);
        });

        const unsubMeta = onSurfaceMetadataChanged((m) => setHostWindowLabel(m.hostWindowLabel));

        onCleanup(() => {
            void unsubMeta.then((d) => d());
        });
    });
}
