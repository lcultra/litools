import { useNavigate } from '@solidjs/router';
import { createContext, createEffect, createSignal, type JSX, onCleanup, onMount, useContext } from 'solid-js';
import { closePluginView, getCurrentSurfaceMetadata, getSettings, hideSurface, updateSurfaceRoute } from './bridge/commands';
import { onNavigate, onSurfaceMetadataChanged } from './bridge/events';
import type { AppSettings } from './bridge/types';
import { isDarkThemeValue } from './shared/theme';
import { isPluginRoutePath, pluginRouteParts } from './views/registry';

// ── shared context ──

type ShellContextValue = {
    hostWindowLabel: () => string | null;
    isDetachedWindow: () => boolean;
    settings: () => AppSettings | null;
};

const ShellCtx = createContext<ShellContextValue>();

export function useShell(): ShellContextValue {
    const ctx = useContext(ShellCtx);
    if (!ctx) throw new Error('useShell must be used inside AppShell');
    return ctx;
}

// ── layout ──

export function AppShell(props: { children?: JSX.Element }) {
    const navigate = useNavigate();
    const [settings, setSettings] = createSignal<AppSettings | null>(null);
    const [systemDark, setSystemDark] = createSignal(false);
    const [hostWindowLabel, setHostWindowLabel] = createSignal<string | null>(null);
    const isDetachedWindow = () => Boolean(hostWindowLabel() && hostWindowLabel() !== 'main');

    onMount(() => {
        void refreshSettings();
        void restoreSurfaceHost();

        const media = window.matchMedia('(prefers-color-scheme: dark)');
        setSystemDark(media.matches);
        const handleSystemTheme = (event: MediaQueryListEvent) => setSystemDark(event.matches);
        media.addEventListener('change', handleSystemTheme);

        const unsubscribe = onNavigate((path) => safeNavigate(path));
        const unsubscribeSurfaceMetadata = onSurfaceMetadataChanged((metadata) => {
            setHostWindowLabel(metadata.hostWindowLabel);
        });
        const handleKeyDown = (event: KeyboardEvent) => {
            if (event.key !== 'Escape' || location.hash === '#/') return;
            event.preventDefault();
            closeCurrentView();
        };
        window.addEventListener('keydown', handleKeyDown);

        function preventContextMenu(event: MouseEvent) {
            event.preventDefault();
        }
        window.addEventListener('contextmenu', preventContextMenu);

        onCleanup(() => {
            media.removeEventListener('change', handleSystemTheme);
            window.removeEventListener('keydown', handleKeyDown);
            window.removeEventListener('contextmenu', preventContextMenu);
            void unsubscribe.then((dispose) => dispose());
            void unsubscribeSurfaceMetadata.then((dispose) => dispose());
        });
    });

    createEffect(() => {
        document.documentElement.classList.toggle('dark', isDarkTheme());
    });

    createEffect(() => {
        const hash = location.hash.slice(1);
        if (!hostWindowLabel() || hash === '/' || (hash === '/' && isDetachedWindow())) return;
        if (isPluginRoutePath(hash)) {
            void updateSurfaceRoute(hash);
        }
    });

    async function refreshSettings() {
        setSettings(await getSettings());
    }

    async function restoreSurfaceHost() {
        const metadata = await getCurrentSurfaceMetadata();
        setHostWindowLabel(metadata?.hostWindowLabel ?? 'main');
    }

    function isDarkTheme() {
        return isDarkThemeValue(settings()?.theme, systemDark());
    }

    function safeNavigate(path: string) {
        if (path === '/' && isDetachedWindow()) return;
        navigate(path);
    }

    function closeCurrentView() {
        const hash = location.hash.slice(1);
        const parts = pluginRouteParts(hash);
        if (parts) {
            void closePluginView(parts.pluginId, parts.commandId);
            navigate('/');
            return;
        }
        if (isDetachedWindow()) {
            void hideSurface();
            return;
        }
        navigate('/');
    }

    return (
        <ShellCtx.Provider value={{ hostWindowLabel, isDetachedWindow, settings }}>
            <main class="h-screen overflow-hidden text-fg transition-colors">{props.children}</main>
        </ShellCtx.Provider>
    );
}
