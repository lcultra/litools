import { Button } from '@litools/ui';
import type { JSX } from 'solid-js';
import { Show } from 'solid-js';

type PageStateVariant = 'empty' | 'error' | 'loading';

type PageStateProps = {
    action?: {
        label: string;
        onClick: () => void;
    };
    children?: JSX.Element;
    description?: string;
    title: string;
    variant?: PageStateVariant;
};

const variantClasses: Record<PageStateVariant, string> = {
    empty: 'text-text-muted',
    error: 'text-danger',
    loading: 'text-text-muted',
};

export function PageState(props: PageStateProps) {
    const variant = () => props.variant ?? 'empty';

    return (
        <div class="rounded-xl bg-surface-hover px-4 py-6 text-center text-sm">
            <p class="m-0 font-semibold" classList={{ [variantClasses[variant()]]: true }}>
                {props.title}
            </p>
            <Show when={props.description}>{(description) => <p class="m-0 mt-2 text-text-muted">{description()}</p>}</Show>
            {props.children}
            <Show when={props.action}>
                {(action) => (
                    <div class="mt-4">
                        <Button onClick={action().onClick} variant="secondary">
                            {action().label}
                        </Button>
                    </div>
                )}
            </Show>
        </div>
    );
}
