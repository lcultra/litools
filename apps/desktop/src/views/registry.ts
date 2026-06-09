export const viewKinds = ['launcher', 'plugin'] as const;
export const hostKinds = ['main', 'detached', 'runtime'] as const;

export type ViewKind = (typeof viewKinds)[number];
export type HostKind = (typeof hostKinds)[number];

export const viewIds = ['palette', 'plugin', 'titlebar'] as const;

export type AppViewId = (typeof viewIds)[number];
export type CoreRoutePath = '/';
export type PluginRoutePath = `/plugin/${string}/${string}`;
export type TitlebarRoutePath = `/titlebar/${string}`;
export type AppRoutePath = CoreRoutePath | PluginRoutePath | TitlebarRoutePath;

type RouteDefinition = {
    allowedHosts: HostKind[];
    defaultHost: HostKind;
    description: string;
    id: AppViewId;
    kind: ViewKind;
    label: string;
    path: AppRoutePath;
    title: string;
};

export const routeDefinitions: RouteDefinition[] = [
    {
        allowedHosts: ['main'],
        defaultHost: 'main',
        description: '搜索命令、本地应用和插件功能。',
        id: 'palette',
        kind: 'launcher',
        label: '启动器',
        path: '/',
        title: '启动器',
    },
];

export const fallbackRoute = routeDefinitions[0];

export function isAppViewId(value: string): value is AppViewId {
    return viewIds.includes(value as AppViewId);
}

export function isPluginRoutePath(value: string): value is PluginRoutePath {
    const parts = value.split('/');
    return parts.length === 4 && parts[0] === '' && parts[1] === 'plugin' && Boolean(parts[2]) && Boolean(parts[3]);
}

export function isTitlebarRoutePath(value: string): value is TitlebarRoutePath {
    const parts = value.split('/');
    return parts.length === 3 && parts[0] === '' && parts[1] === 'titlebar' && Boolean(parts[2]);
}

export function isAppRoutePath(value: string): value is AppRoutePath {
    return routeDefinitions.some((route) => route.path === value) || isPluginRoutePath(value) || isTitlebarRoutePath(value);
}

export function routeForViewId(viewId: AppViewId) {
    return routeDefinitions.find((route) => route.id === viewId) ?? fallbackRoute;
}

export function routeForPath(pathname: string): RouteDefinition {
    if (isPluginRoutePath(pathname)) {
        return {
            allowedHosts: ['main', 'detached', 'runtime'],
            defaultHost: 'runtime',
            description: '运行插件视图。',
            id: 'plugin',
            kind: 'plugin',
            label: '插件',
            path: pathname,
            title: '插件',
        };
    }

    if (isTitlebarRoutePath(pathname)) {
        return {
            allowedHosts: ['runtime'],
            defaultHost: 'runtime',
            description: '分离窗口标题栏。',
            id: 'titlebar',
            kind: 'plugin',
            label: '标题栏',
            path: pathname,
            title: '标题栏',
        };
    }

    return routeDefinitions.find((route) => route.path === pathname) ?? fallbackRoute;
}

export function isRouteAllowedInHost(path: AppRoutePath, host: HostKind) {
    return routeForPath(path).allowedHosts.includes(host);
}

export function hostForRoute(path: AppRoutePath) {
    return routeForPath(path).defaultHost;
}

export function canDetachRoute(path: AppRoutePath) {
    const route = routeForPath(path);
    return route.kind !== 'launcher' && route.allowedHosts.includes('detached');
}

export function pathForNavigationPayload(payload: string): AppRoutePath | null {
    return isAppRoutePath(payload) ? payload : null;
}

export function pluginRoute(pluginId: string, commandId: string): PluginRoutePath {
    return `/plugin/${pluginId}/${commandId}`;
}

export function pluginRouteParts(path: AppRoutePath): { pluginId: string; commandId: string } | null {
    if (!isPluginRoutePath(path)) {
        return null;
    }

    const [, , pluginId, commandId] = path.split('/');
    return { pluginId, commandId };
}

export function titlebarRouteParts(path: AppRoutePath): { runtimeId: string } | null {
    if (!isTitlebarRoutePath(path)) {
        return null;
    }

    const [, , runtimeId] = path.split('/');
    return { runtimeId };
}
