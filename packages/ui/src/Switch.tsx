import { Switch as ArkSwitch } from '@ark-ui/solid';

type SwitchProps = {
    checked: boolean;
    onChange: (checked: boolean) => void;
    disabled?: boolean;
    label?: string;
};

export function Switch(props: SwitchProps) {
    return (
        <ArkSwitch.Root
            checked={props.checked}
            onCheckedChange={(e) => props.onChange(e.checked)}
            disabled={props.disabled}
            class="inline-flex items-center gap-2"
        >
            <ArkSwitch.Control class="relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full
              border-2 border-transparent transition-colors
              bg-border data-[checked]:bg-accent">
                <ArkSwitch.Thumb class="pointer-events-none block h-4 w-4 rounded-full bg-white
                  shadow transition-transform data-[checked]:translate-x-4 translate-x-0" />
            </ArkSwitch.Control>
            {props.label && (
                <ArkSwitch.Label class="text-sm text-fg">{props.label}</ArkSwitch.Label>
            )}
        </ArkSwitch.Root>
    );
}
