export const STRINGS = {
    provider: {
        apps: '应用',
        commands: '命令',
        plugins: '插件',
    } as Record<string, string>,
};

export function providerLabel(provider: string) {
    return STRINGS.provider[provider] ?? provider;
}
