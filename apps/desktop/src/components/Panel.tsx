import type { JSX } from 'solid-js';

type PanelProps = {
    children: JSX.Element;
    class?: string;
    variant?: 'launcher' | 'page';
};

export function Panel(props: PanelProps) {
    const baseClass = 'overflow-hidden rounded-[20px] bg-surface border border-border';
    const variantClass = () => (props.variant === 'launcher' ? 'backdrop-blur' : 'p-6');

    return <section class={[baseClass, variantClass(), props.class].filter(Boolean).join(' ')}>{props.children}</section>;
}
