import type { ThemeValue } from '@litools/plugin-core';

export type { ThemeValue };

export function isDarkThemeValue(theme: string | null | undefined, systemDark: boolean) {
    return theme === 'dark' || (theme === 'system' && systemDark);
}
