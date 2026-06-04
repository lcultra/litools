export function providerLabel(provider: string) {
    if (provider === 'commands') {
        return '命令';
    }

    if (provider === 'apps') {
        return '应用';
    }

    return provider;
}

export function targetTypeLabel(targetType: string) {
    if (targetType === 'command') {
        return '命令';
    }

    if (targetType === 'app') {
        return '应用';
    }

    return targetType;
}
