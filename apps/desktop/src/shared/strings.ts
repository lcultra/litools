export const STRINGS = {
    provider: {
        apps: '应用',
        commands: '命令',
        plugins: '插件',
    } as Record<string, string>,

    launcher: {
        emptyResult: '未找到结果',
        inputPlaceholder: '搜索应用、命令、文件、插件...',
    },

    plugin: {
        loading: '正在加载插件...',
    },
};

export function providerLabel(provider: string) {
    return STRINGS.provider[provider] ?? provider;
}
