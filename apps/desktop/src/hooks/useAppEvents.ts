import { onCleanup, onMount } from 'solid-js';
import { getCurrentSurfaceMetadata, getSettings } from '../bridge/commands';
import { onNavigate, onSurfaceMetadataChanged } from '../bridge/events';
import { setHostWindowLabel, setSettings } from '../shared/store';

export function useAppEvents() {
    onMount(() => {
        void getCurrentSurfaceMetadata().then((m) => setHostWindowLabel(m?.hostWindowLabel ?? 'main'));
        void getSettings().then(setSettings);

        const unsubNav = onNavigate(() => {
            // Backend navigation events are validated but not acted on;
            // the HashRouter drives actual route changes.
        });
        const unsubMeta = onSurfaceMetadataChanged((m) => setHostWindowLabel(m.hostWindowLabel));

        onCleanup(() => {
            void unsubNav.then((d) => d());
            void unsubMeta.then((d) => d());
        });
    });
}
