import { onCleanup, onMount } from 'solid-js';

/**
 * 绑定全局键盘快捷键。
 * 在组件 onMount 时注册 window keydown 监听，onCleanup 时自动移除。
 */
export function useGlobalKey(key: string, handler: (event: KeyboardEvent) => void, options?: { prevent?: boolean; stop?: boolean }): void {
    onMount(() => {
        const listener = (e: KeyboardEvent) => {
            if (e.key !== key) return;
            if (options?.prevent) e.preventDefault();
            if (options?.stop) e.stopPropagation();
            handler(e);
        };
        window.addEventListener('keydown', listener);
        onCleanup(() => window.removeEventListener('keydown', listener));
    });
}
