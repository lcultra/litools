import type { ThemeValue } from '@litools/plugin-core';

export type { ThemeValue };

export type ThemeOption = {
    label: string;
    value: ThemeValue;
};

export const themeOptions: ThemeOption[] = [
    { label: '跟随系统', value: 'system' },
    { label: '浅色', value: 'light' },
    { label: '深色', value: 'dark' },
];

export function isThemeValue(value: string): value is ThemeValue {
    return value === 'system' || value === 'light' || value === 'dark';
}

export function themeLabel(theme: string) {
    if (theme === 'dark') {
        return '深色';
    }

    if (theme === 'light') {
        return '浅色';
    }

    return '跟随系统';
}

export function isDarkThemeValue(theme: string | null | undefined, systemDark: boolean) {
    return theme === 'dark' || (theme === 'system' && systemDark);
}
