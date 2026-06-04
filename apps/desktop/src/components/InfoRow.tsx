import type { JSX } from 'solid-js';

type InfoRowProps = {
    children?: JSX.Element;
    label: string;
    value?: string;
};

export function InfoRow(props: InfoRowProps) {
    return (
        <div class="grid gap-1 rounded-xl bg-surface-muted px-4 py-3 sm:flex sm:items-center sm:justify-between sm:gap-4">
            <span class="text-muted">{props.label}</span>
            {props.children ?? <span class="break-all font-medium">{props.value}</span>}
        </div>
    );
}
