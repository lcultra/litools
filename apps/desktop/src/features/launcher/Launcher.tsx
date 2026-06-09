import { useNavigate } from '@solidjs/router';
import type { CommandEffect } from '../../bridge/types';
import { LauncherController } from './LauncherController';

export function Launcher() {
    const navigate = useNavigate();

    function handleCommandEffect(effect: CommandEffect) {
        if (typeof effect === 'object' && 'openPluginView' in effect) {
            navigate(effect.openPluginView.route);
        }
    }

    return <LauncherController onCommandEffect={handleCommandEffect} />;
}
