import { Switch as KobalteSwitch } from '@kobalte/core/switch';

interface SwitchProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
  label?: string;
}

export function Switch(props: SwitchProps) {
  return (
    <KobalteSwitch
      checked={props.checked}
      onChange={props.onChange}
      disabled={props.disabled}
      class="inline-flex items-center gap-2"
    >
      <KobalteSwitch.Input />
      <KobalteSwitch.Control class="inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full bg-border ring-1 ring-inset ring-black/10 transition-colors data-checked:bg-primary data-disabled:opacity-40 data-disabled:cursor-not-allowed">
        <KobalteSwitch.Thumb class="block h-4 w-4 rounded-full bg-white shadow ring-1 ring-black/10 transition-transform data-checked:translate-x-4.5 translate-x-0.5" />
      </KobalteSwitch.Control>
      {props.label && (
        <KobalteSwitch.Label class="text-sm text-text select-none">
          {props.label}
        </KobalteSwitch.Label>
      )}
    </KobalteSwitch>
  );
}
