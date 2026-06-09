import { useNavigate } from '@solidjs/router';
import type { JSX } from 'solid-js';
import { createEffect, onCleanup, onMount } from 'solid-js';
import { closePluginView, hideSurface } from './bridge/commands';
import { useAppEvents } from './hooks/useAppEvents';
import { usePluginMatch } from './hooks/usePluginMatch';
import { isDetachedWindow, settings } from './shared/store';
import { isDarkThemeValue } from './shared/theme';

export function AppLayout(props: { children?: JSX.Element }) {
    const navigate = useNavigate();
    const pluginMatch = usePluginMatch();

    // Tauri event listeners → store
    useAppEvents();

    // Theme
    createEffect(() => {
        const mq = window.matchMedia('(prefers-color-scheme: dark)');
        const update = () => document.documentElement.classList.toggle('dark', isDarkThemeValue(settings()?.theme, mq.matches));
        update();
        const handler = () => update();
        mq.addEventListener('change', handler);
        onCleanup(() => mq.removeEventListener('change', handler));
    });

    // Keyboard
    onMount(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key !== 'Escape' || location.hash === '#/') return;
            e.preventDefault();
            closeCurrentView();
        };
        const preventCtx = (e: MouseEvent) => e.preventDefault();
        window.addEventListener('keydown', handleKeyDown);
        window.addEventListener('contextmenu', preventCtx);
        onCleanup(() => {
            window.removeEventListener('keydown', handleKeyDown);
            window.removeEventListener('contextmenu', preventCtx);
        });
    });

    function closeCurrentView() {
        const match = pluginMatch();
        if (match) {
            void closePluginView(match.pluginId, match.commandId);
            navigate('/');
            return;
        }
        if (isDetachedWindow()) {
            void hideSurface();
            return;
        }
        navigate('/');
    }

    return <main class="h-screen overflow-hidden text-fg transition-colors">{props.children}</main>;
}
