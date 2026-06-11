import type { ParentComponent } from 'solid-js';

type BadgeProps = {
    variant?: 'default' | 'success' | 'warning' | 'bundled' | 'user';
    size?: 'sm' | 'md';
};

const variantClass: Record<string, string> = {
    default: 'bg-bg-muted text-fg-muted',
    success: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
    warning: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200',
    bundled: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
    user: 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200',
};

export const Badge: ParentComponent<BadgeProps> = (props) => {
    return (
        <span
            class={`inline-flex items-center rounded-full font-medium
              ${variantClass[props.variant ?? 'default']}
              ${props.size === 'sm' ? 'px-1.5 py-0.5 text-[10px]' : 'px-2 py-0.5 text-xs'}`}
        >
            {props.children}
        </span>
    );
};
