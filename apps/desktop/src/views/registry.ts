export const viewKinds = ['launcher', 'panel', 'runtime'] as const;
export const hostKinds = ['main', 'detached', 'runtime'] as const;
export const managementNavGroups = ['general', 'plugins', 'diagnostics'] as const;

export type ViewKind = (typeof viewKinds)[number];
export type HostKind = (typeof hostKinds)[number];
export type ManagementNavGroupId = (typeof managementNavGroups)[number];

export const viewIds = ['palette', 'settings', 'diagnostics', 'plugins', 'pluginRuntime'] as const;

export type AppViewId = (typeof viewIds)[number];
export type CoreRoutePath = '/' | '/settings' | '/diagnostics' | '/plugins';
export type PluginRuntimeRoutePath = `/plugin-runtime/${string}/${string}`;
export type AppRoutePath = CoreRoutePath | PluginRuntimeRoutePath;

type RouteDefinition = {
    allowedHosts: HostKind[];
    defaultHost: HostKind;
    description: string;
    id: AppViewId;
    kind: ViewKind;
    label: string;
    navGroup?: ManagementNavGroupId;
    path: AppRoutePath;
    showInManagementNav: boolean;
    title: string;
};

export type ManagementNavGroup = {
    id: ManagementNavGroupId;
    label: string;
};

export type ManagementNavItem = Pick<RouteDefinition, 'description' | 'id' | 'label' | 'navGroup' | 'path' | 'title'>;

export const routeDefinitions: RouteDefinition[] = [
    {
        allowedHosts: ['main'],
        defaultHost: 'main',
        description: '搜索命令、本地应用和未来插件功能。',
        id: 'palette',
        kind: 'launcher',
        label: '启动器',
        path: '/',
        showInManagementNav: false,
        title: '启动器',
    },
    {
        allowedHosts: ['main', 'detached'],
        defaultHost: 'main',
        description: '配置命令面板运行参数。',
        id: 'settings',
        kind: 'panel',
        label: '设置',
        navGroup: 'general',
        path: '/settings',
        showInManagementNav: true,
        title: '设置',
    },
    {
        allowedHosts: ['main', 'detached'],
        defaultHost: 'main',
        description: '管理插件和扩展能力。',
        id: 'plugins',
        kind: 'panel',
        label: '插件中心',
        navGroup: 'plugins',
        path: '/plugins',
        showInManagementNav: true,
        title: '插件中心',
    },
    {
        allowedHosts: ['main', 'detached'],
        defaultHost: 'main',
        description: '查看 litools 的运行状态和本地数据。',
        id: 'diagnostics',
        kind: 'panel',
        label: '运行状态',
        navGroup: 'diagnostics',
        path: '/diagnostics',
        showInManagementNav: true,
        title: '诊断',
    },
];

export const managementNavGroupsById: Record<ManagementNavGroupId, ManagementNavGroup> = {
    diagnostics: { id: 'diagnostics', label: '诊断' },
    general: { id: 'general', label: '常规' },
    plugins: { id: 'plugins', label: '插件' },
};

export const managementNavGroupList: ManagementNavGroup[] = managementNavGroups.map((id) => managementNavGroupsById[id]);

export const managementNavItems: ManagementNavItem[] = routeDefinitions.filter((route) => route.showInManagementNav);

export const fallbackRoute = routeDefinitions[0];

export function isAppViewId(value: string): value is AppViewId {
    return viewIds.includes(value as AppViewId);
}

export function isPluginRuntimeRoutePath(value: string): value is PluginRuntimeRoutePath {
    const parts = value.split('/');
    return parts.length === 4 && parts[0] === '' && parts[1] === 'plugin-runtime' && Boolean(parts[2]) && Boolean(parts[3]);
}

export function isAppRoutePath(value: string): value is AppRoutePath {
    return routeDefinitions.some((route) => route.path === value) || isPluginRuntimeRoutePath(value);
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
            showInManagementNav: false,
            title: '插件运行时',
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
