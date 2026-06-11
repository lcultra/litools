import { onCleanup, onMount } from 'solid-js';

/**
 * 绑定全局键盘快捷键。
 * 在组件 onMount 时注册 window keydown 监听，onCleanup 时自动移除。
 *
 * options.modifier - 可选修饰键：'meta' (Cmd/Windows)、'ctrl'、'alt'。
 *                    不传则匹配无修饰键的单独按键。
 */
export function useGlobalKey(key: string, handler: (event: KeyboardEvent) => void, options?: { prevent?: boolean; stop?: boolean; modifier?: 'meta' | 'ctrl' | 'alt' }): void {
    onMount(() => {
        const listener = (e: KeyboardEvent) => {
            if (e.key !== key) return;
            if (options?.modifier === 'meta' && !e.metaKey) return;
            if (options?.modifier === 'ctrl' && !e.ctrlKey) return;
            if (options?.modifier === 'alt' && !e.altKey) return;
            if (options?.prevent) e.preventDefault();
            if (options?.stop) e.stopPropagation();
            handler(e);
        };
        window.addEventListener('keydown', listener);
        onCleanup(() => window.removeEventListener('keydown', listener));
    });
}
