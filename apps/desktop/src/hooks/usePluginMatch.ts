import { useMatch } from '@solidjs/router';
import { PLUGIN_ROUTE_PATTERN } from '../shared/routes';

export type PluginMatchInfo = {
    pluginId: string;
    commandId: string;
} | null;

/** Returns the current plugin's id and command if on a plugin page, null otherwise. */
export function usePluginMatch() {
    const match = useMatch(() => PLUGIN_ROUTE_PATTERN);
    return () => {
        const m = match();
        if (m?.params.pluginId && m.params.commandId) {
            return { pluginId: m.params.pluginId, commandId: m.params.commandId };
        }
        return null;
    };
}
