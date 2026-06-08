import type { CommandEffect } from '../../bridge/types';
import { PaletteSearchController } from './PaletteSearchController';

type CommandPaletteProps = {
    onCommandEffect: (effect: CommandEffect) => void;
};

export function CommandPalette(props: CommandPaletteProps) {
    return <PaletteSearchController onCommandEffect={props.onCommandEffect} />;
}
