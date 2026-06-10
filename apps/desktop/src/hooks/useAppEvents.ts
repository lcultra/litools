import { onCleanup, onMount } from 'solid-js';
import { getCurrentSurfaceMetadata, getSettings } from '../bridge/commands';
import { onSurfaceMetadataChanged } from '../bridge/events';
import { setHostWindowLabel, setSettings } from '../shared/store';

export function useAppEvents() {
    onMount(() => {
        void getCurrentSurfaceMetadata().then((m) => setHostWindowLabel(m?.hostWindowLabel ?? 'main'));
        void getSettings().then(setSettings);

        const unsubMeta = onSurfaceMetadataChanged((m) => setHostWindowLabel(m.hostWindowLabel));

        onCleanup(() => {
            void unsubMeta.then((d) => d());
        });
    });
}
