import type { JSX } from 'solid-js';

type ButtonVariant = 'primary' | 'secondary' | 'ghost';

type ButtonProps = JSX.ButtonHTMLAttributes<HTMLButtonElement> & {
    variant?: ButtonVariant;
};

const variantClasses: Record<ButtonVariant, string> = {
    ghost: 'text-muted hover:bg-surface-muted/60 hover:text-fg focus-visible:bg-surface-muted/60 focus-visible:text-fg',
    primary: 'bg-accent text-accent-fg hover:opacity-90 disabled:opacity-50',
    secondary: 'bg-surface-muted text-fg hover:bg-surface-muted/80 disabled:opacity-50',
};

export function Button(props: ButtonProps) {
    const variant = () => props.variant ?? 'secondary';

    return (
        <button
            {...props}
            class={['rounded-lg px-4 py-2 text-sm font-semibold outline-none transition-colors disabled:pointer-events-none', variantClasses[variant()], props.class]
                .filter(Boolean)
                .join(' ')}
            type={props.type ?? 'button'}
        >
            {props.children}
        </button>
    );
}
