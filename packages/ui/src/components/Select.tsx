import { Select as KobalteSelect } from '@kobalte/core/select';
import type { SelectRootItemComponentProps } from '@kobalte/core/select';
import { ChevronDown } from 'lucide-solid';

interface SelectItem {
  value: string;
  label: string;
}

interface SelectProps {
  label?: string;
  items: SelectItem[];
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  disabled?: boolean;
  required?: boolean;
  invalid?: boolean;
  class?: string;
  contentClass?: string;
}

const Item = (p: SelectRootItemComponentProps<SelectItem>) => (
  <KobalteSelect.Item
    item={p.item}
    class="flex items-center px-3 py-1.5 text-sm text-text cursor-pointer data-highlighted:bg-surface-card data-selected:font-medium rounded-sm outline-none"
  >
    <KobalteSelect.ItemLabel>{p.item.rawValue.label}</KobalteSelect.ItemLabel>
  </KobalteSelect.Item>
);

export function Select(props: SelectProps) {
  const selected = () =>
    props.items.find((i) => i.value === props.value) ?? null;

  return (
    <KobalteSelect<SelectItem>
      options={props.items}
      optionValue="value"
      optionTextValue="label"
      value={selected()}
      onChange={(item) => props.onChange(item?.value ?? '')}
      placeholder={props.placeholder ?? '请选择'}
      disabled={props.disabled}
      validationState={props.invalid ? 'invalid' : 'valid'}
      itemComponent={Item}
      class={props.class}
    >
      {props.label && (
        <KobalteSelect.Label class="block text-sm font-medium text-text mb-1">
          {props.label}
        </KobalteSelect.Label>
      )}
      <KobalteSelect.Trigger
        class="inline-flex items-center justify-between w-full px-3 py-1.5 h-8 rounded-md border border-border bg-surface-card text-text text-sm
          hover:bg-surface-hover
          data-expanded:ring-1 data-expanded:ring-ring
          focus-visible:ring-1 focus-visible:ring-ring outline-none
          data-placeholder-shown:text-text-muted
          data-disabled:opacity-40 data-disabled:cursor-not-allowed"
      >
        <KobalteSelect.Value<SelectItem>>
          {(state) =>
            state.selectedOption()?.label ??
            (props.placeholder ?? '请选择')
          }
        </KobalteSelect.Value>
        <KobalteSelect.Icon class="text-text-muted ml-2 data-expanded:text-text transition-colors">
          <ChevronDown class="w-4 h-4" />
        </KobalteSelect.Icon>
      </KobalteSelect.Trigger>
      <KobalteSelect.Portal>
        <KobalteSelect.Content
          class={`bg-surface border border-border rounded-md shadow-lg px-0.5 py-1 z-50 outline-none ${props.contentClass ?? ''}`}
        >
          <KobalteSelect.Listbox class="outline-none" />
        </KobalteSelect.Content>
      </KobalteSelect.Portal>
    </KobalteSelect>
  );
}
