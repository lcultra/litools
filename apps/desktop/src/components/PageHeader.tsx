import type { JSX } from 'solid-js';

type PageHeaderProps = {
    action?: JSX.Element;
    description: string;
    title: string;
};

export function PageHeader(props: PageHeaderProps) {
    return (
        <div class="flex items-start justify-between gap-4">
            <div>
                <h1 class="m-0 text-2xl font-semibold">{props.title}</h1>
                <p class="m-0 mt-2 text-sm text-muted">{props.description}</p>
            </div>
            {props.action}
        </div>
    );
}
