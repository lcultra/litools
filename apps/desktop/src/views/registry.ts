export const viewKinds = ['launcher', 'plugin'] as const;
export const hostKinds = ['main', 'detached', 'runtime'] as const;

export type ViewKind = (typeof viewKinds)[number];
export type HostKind = (typeof hostKinds)[number];

export type AppRoutePath = string;

type RouteDefinition = {
    allowedHosts: HostKind[];
    defaultHost: HostKind;
    kind: ViewKind;
    path: string;
};

const pluginRouteDef: RouteDefinition = {
    allowedHosts: ['main', 'detached', 'runtime'],
    defaultHost: 'runtime',
    kind: 'plugin',
    path: '/plugin/:pluginId/:commandId',
};

export function routeDefForPath(pathname: string): RouteDefinition {
    if (pathname === '/') {
        return { allowedHosts: ['main'], defaultHost: 'main', kind: 'launcher', path: '/' };
    }
    return pluginRouteDef;
}

export function canDetachRoute(path: string) {
    const route = routeDefForPath(path);
    return route.kind !== 'launcher' && route.allowedHosts.includes('detached');
}

export function isPluginRoutePath(value: string): boolean {
    const parts = value.split('/');
    return parts.length === 4 && parts[0] === '' && parts[1] === 'plugin' && Boolean(parts[2]) && Boolean(parts[3]);
}

export function pluginRouteParts(path: string): { pluginId: string; commandId: string } | null {
    if (!isPluginRoutePath(path)) return null;
    const [, , pluginId, commandId] = path.split('/');
    return { pluginId, commandId };
}

export function pluginRoute(pluginId: string, commandId: string): string {
    return `/plugin/${pluginId}/${commandId}`;
}
