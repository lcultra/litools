import type { JSX } from 'solid-js';
import { Show } from 'solid-js';
import { Button } from './Button';

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
    empty: 'text-muted',
    error: 'text-danger',
    loading: 'text-muted',
};

export function PageState(props: PageStateProps) {
    const variant = () => props.variant ?? 'empty';

    return (
        <div class="rounded-xl bg-surface-muted px-4 py-6 text-center text-sm">
            <p class="m-0 font-semibold" classList={{ [variantClasses[variant()]]: true }}>
                {props.title}
            </p>
            <Show when={props.description}>{(description) => <p class="m-0 mt-2 text-muted">{description()}</p>}</Show>
            {props.children}
            <Show when={props.action}>
                {(action) => (
                    <Button class="mt-4" onClick={action().onClick} variant="secondary">
                        {action().label}
                    </Button>
                )}
            </Show>
        </div>
    );
}
