import type { BuiltinCommandEffect } from '../../bridge/types';
import { PaletteSearchController } from './PaletteSearchController';

type CommandPaletteProps = {
    onCommandEffect: (effect: BuiltinCommandEffect) => void;
};

export function CommandPalette(props: CommandPaletteProps) {
    return <PaletteSearchController onCommandEffect={props.onCommandEffect} />;
}
