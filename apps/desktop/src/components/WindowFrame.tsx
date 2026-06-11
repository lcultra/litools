import type { JSX } from 'solid-js';

type WindowFrameProps = {
    children: JSX.Element;
    class?: string;
    ref?: (element: HTMLDivElement) => void;
};

export function WindowFrame(props: WindowFrameProps) {
    return (
        <div ref={props.ref} class="p-px">
            <section class="w-full self-start overflow-hidden rounded-[20px] bg-surface border border-border backdrop-blur" classList={{ [props.class ?? '']: !!props.class }}>
                {props.children}
            </section>
        </div>
    );
}
