export const surfaceKinds = ['launcher', 'runtime', 'management'] as const;
export const windowTargets = ['main', 'management', 'runtime', 'detached'] as const;
export const windowProfiles = ['launcher', 'runtime', 'management'] as const;
export const managementNavGroups = ['general', 'plugins', 'diagnostics'] as const;

export type SurfaceKind = (typeof surfaceKinds)[number];
export type WindowTarget = (typeof windowTargets)[number];
export type WindowProfile = (typeof windowProfiles)[number];
export type ManagementNavGroupId = (typeof managementNavGroups)[number];

export const viewIds = ['palette', 'settings', 'diagnostics', 'plugins'] as const;

export type AppViewId = (typeof viewIds)[number];
export type AppRoutePath = '/' | '/settings' | '/diagnostics' | '/plugins';

type RouteDefinition = {
    allowedWindowTargets: WindowTarget[];
    defaultWindowTarget: WindowTarget;
    description: string;
    id: AppViewId;
    kind: SurfaceKind;
    label: string;
    navGroup?: ManagementNavGroupId;
    path: AppRoutePath;
    showInManagementNav: boolean;
    title: string;
    windowProfile: WindowProfile;
};

export type ManagementNavGroup = {
    id: ManagementNavGroupId;
    label: string;
};

export type ManagementNavItem = Pick<RouteDefinition, 'description' | 'id' | 'label' | 'navGroup' | 'path' | 'title'>;

export const routeDefinitions: RouteDefinition[] = [
    {
        allowedWindowTargets: ['main'],
        defaultWindowTarget: 'main',
        description: '搜索命令、本地应用和未来插件功能。',
        id: 'palette',
        kind: 'launcher',
        label: '启动器',
        path: '/',
        showInManagementNav: false,
        title: '启动器',
        windowProfile: 'launcher',
    },
    {
        allowedWindowTargets: ['main', 'management'],
        defaultWindowTarget: 'main',
        description: '配置命令面板运行参数。',
        id: 'settings',
        kind: 'management',
        label: '设置',
        navGroup: 'general',
        path: '/settings',
        showInManagementNav: true,
        title: '设置',
        windowProfile: 'management',
    },
    {
        allowedWindowTargets: ['main', 'management'],
        defaultWindowTarget: 'main',
        description: '管理插件和扩展能力。',
        id: 'plugins',
        kind: 'management',
        label: '插件中心',
        navGroup: 'plugins',
        path: '/plugins',
        showInManagementNav: true,
        title: '插件中心',
        windowProfile: 'management',
    },
    {
        allowedWindowTargets: ['main', 'management'],
        defaultWindowTarget: 'main',
        description: '查看 litools 的运行状态和本地数据。',
        id: 'diagnostics',
        kind: 'management',
        label: '运行状态',
        navGroup: 'diagnostics',
        path: '/diagnostics',
        showInManagementNav: true,
        title: '诊断',
        windowProfile: 'management',
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

export function isAppRoutePath(value: string): value is AppRoutePath {
    return routeDefinitions.some((route) => route.path === value);
}

export function routeForViewId(viewId: AppViewId) {
    return routeDefinitions.find((route) => route.id === viewId) ?? fallbackRoute;
}

export function routeForPath(pathname: string) {
    return routeDefinitions.find((route) => route.path === pathname) ?? fallbackRoute;
}

export function pathForNavigationPayload(payload: string): AppRoutePath | null {
    return isAppRoutePath(payload) ? payload : null;
}
