import type { ParentComponent, JSX } from 'solid-js';

export const Card: ParentComponent<{
    onClick?: JSX.EventHandler<HTMLDivElement, MouseEvent>;
    class?: string;
}> = (props) => {
    return (
        <div
            onClick={props.onClick}
            class={`rounded-lg border border-border bg-bg p-4
              ${props.onClick ? 'cursor-pointer hover:bg-bg-muted transition-colors' : ''}
              ${props.class ?? ''}`}
        >
            {props.children}
        </div>
    );
};
