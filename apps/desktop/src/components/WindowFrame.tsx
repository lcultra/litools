import type { JSX } from 'solid-js';
import { Panel } from './Panel';

type WindowFrameProps = {
    children: JSX.Element;
    class?: string;
    ref?: (element: HTMLDivElement) => void;
};

export function WindowFrame(props: WindowFrameProps) {
    return (
        <div ref={props.ref} class="p-px">
            <Panel class={['w-full self-start', props.class].filter(Boolean).join(' ')} variant="launcher">
                {props.children}
            </Panel>
        </div>
    );
}
