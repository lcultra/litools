export const viewKinds = ['launcher', 'runtime'] as const;
export const hostKinds = ['main', 'detached', 'runtime'] as const;

export type ViewKind = (typeof viewKinds)[number];
export type HostKind = (typeof hostKinds)[number];

export const viewIds = ['palette', 'pluginRuntime', 'pluginRuntimeHeader'] as const;

export type AppViewId = (typeof viewIds)[number];
export type CoreRoutePath = '/';
export type PluginRuntimeRoutePath = `/plugin-runtime/${string}/${string}`;
export type PluginRuntimeHeaderRoutePath = `/plugin-runtime-header/${string}`;
export type AppRoutePath = CoreRoutePath | PluginRuntimeRoutePath | PluginRuntimeHeaderRoutePath;

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

export function isPluginRuntimeRoutePath(value: string): value is PluginRuntimeRoutePath {
    const parts = value.split('/');
    return parts.length === 4 && parts[0] === '' && parts[1] === 'plugin-runtime' && Boolean(parts[2]) && Boolean(parts[3]);
}

export function isPluginRuntimeHeaderRoutePath(value: string): value is PluginRuntimeHeaderRoutePath {
    const parts = value.split('/');
    return parts.length === 3 && parts[0] === '' && parts[1] === 'plugin-runtime-header' && Boolean(parts[2]);
}

export function isAppRoutePath(value: string): value is AppRoutePath {
    return routeDefinitions.some((route) => route.path === value) || isPluginRuntimeRoutePath(value) || isPluginRuntimeHeaderRoutePath(value);
}

export function routeForViewId(viewId: AppViewId) {
    return routeDefinitions.find((route) => route.id === viewId) ?? fallbackRoute;
}

export function routeForPath(pathname: string): RouteDefinition {
    if (isPluginRuntimeRoutePath(pathname)) {
        return {
            allowedHosts: ['main', 'detached', 'runtime'],
            defaultHost: 'runtime',
            description: '运行插件视图。',
            id: 'pluginRuntime',
            kind: 'runtime',
            label: '插件运行时',
            path: pathname,
            title: '插件运行时',
        };
    }

    if (isPluginRuntimeHeaderRoutePath(pathname)) {
        return {
            allowedHosts: ['runtime'],
            defaultHost: 'runtime',
            description: '插件运行时标题栏。',
            id: 'pluginRuntimeHeader',
            kind: 'runtime',
            label: '插件标题栏',
            path: pathname,
            title: '插件标题栏',
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
    return route.kind !== 'launcher' && route.id !== 'pluginRuntimeHeader' && route.allowedHosts.includes('detached');
}

export function pathForNavigationPayload(payload: string): AppRoutePath | null {
    return isAppRoutePath(payload) ? payload : null;
}

export function pluginRuntimeRoute(pluginId: string, commandId: string): PluginRuntimeRoutePath {
    return `/plugin-runtime/${pluginId}/${commandId}`;
}

export function pluginRuntimeRouteParts(path: AppRoutePath): { pluginId: string; commandId: string } | null {
    if (!isPluginRuntimeRoutePath(path)) {
        return null;
    }

    const [, , pluginId, commandId] = path.split('/');
    return { pluginId, commandId };
}

export function pluginRuntimeHeaderRouteParts(path: AppRoutePath): { runtimeId: string } | null {
    if (!isPluginRuntimeHeaderRoutePath(path)) {
        return null;
    }

    const [, , runtimeId] = path.split('/');
    return { runtimeId };
}
