import type { ParentComponent, JSX } from 'solid-js';

type ButtonProps = {
    variant?: 'primary' | 'secondary' | 'danger' | 'ghost';
    size?: 'sm' | 'md';
    disabled?: boolean;
    onClick?: JSX.EventHandler<HTMLButtonElement, MouseEvent>;
};

const variantClass: Record<string, string> = {
    primary: 'bg-accent text-white hover:bg-accent-hover',
    secondary: 'bg-bg-muted text-fg hover:bg-border',
    danger: 'bg-red-600 text-white hover:bg-red-700',
    ghost: 'text-fg-muted hover:bg-bg-muted',
};

const sizeClass: Record<string, string> = {
    sm: 'px-2 py-1 text-xs rounded',
    md: 'px-3 py-1.5 text-sm rounded-md',
};

export const Button: ParentComponent<ButtonProps> = (props) => {
    return (
        <button
            type="button"
            disabled={props.disabled}
            onClick={props.onClick}
            class={`inline-flex items-center gap-1.5 font-medium transition-colors
              disabled:opacity-40 disabled:cursor-not-allowed
              ${variantClass[props.variant ?? 'secondary']}
              ${sizeClass[props.size ?? 'md']}`}
        >
            {props.children}
        </button>
    );
};
