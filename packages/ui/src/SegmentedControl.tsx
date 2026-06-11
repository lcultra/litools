import { For } from 'solid-js';

export type SegmentedOption<T extends string> = {
    value: T;
    label: string;
};

type SegmentedControlProps<T extends string> = {
    options: SegmentedOption<T>[];
    value: T;
    onChange: (value: T) => void;
};

export function SegmentedControl<T extends string>(props: SegmentedControlProps<T>) {
    return (
        <div
            class="inline-flex rounded-lg border border-border bg-bg-muted p-0.5"
            role="radiogroup"
        >
            <For each={props.options}>
                {(option) => (
                    <button
                        type="button"
                        role="radio"
                        aria-checked={option.value === props.value}
                        onClick={() => props.onChange(option.value)}
                        class={`px-3 py-1.5 text-sm font-medium rounded-md transition-colors
                          ${option.value === props.value
                            ? 'bg-bg text-fg shadow-sm'
                            : 'text-fg-muted hover:text-fg'}`}
                    >
                        {option.label}
                    </button>
                )}
            </For>
        </div>
    );
}
