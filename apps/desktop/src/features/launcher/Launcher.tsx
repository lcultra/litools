import type { CommandEffect } from '../../bridge/types';
import { LauncherController } from './LauncherController';

type LauncherProps = {
    onCommandEffect: (effect: CommandEffect) => void;
};

export function Launcher(props: LauncherProps) {
    return <LauncherController onCommandEffect={props.onCommandEffect} />;
}
