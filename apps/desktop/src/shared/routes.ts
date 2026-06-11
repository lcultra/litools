export const PLUGIN_ROUTE_PATTERN = '/plugin/:pluginId/:commandId';

export function generatePluginPath(pluginId: string, commandId: string): string {
    return `/plugin/${pluginId}/${commandId}`;
}
