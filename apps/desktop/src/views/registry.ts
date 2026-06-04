export const viewIds = ['palette', 'settings', 'diagnostics', 'plugins'] as const;

export type AppViewId = (typeof viewIds)[number];

export type ViewNavItem = {
    id: AppViewId;
    label: string;
};

export const secondaryViewNavItems: ViewNavItem[] = [
    { id: 'palette', label: '启动器' },
    { id: 'settings', label: '设置' },
    { id: 'diagnostics', label: '诊断' },
    { id: 'plugins', label: '插件' },
];

export function isAppViewId(value: string): value is AppViewId {
    return viewIds.includes(value as AppViewId);
}
