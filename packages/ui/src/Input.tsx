import type { JSX } from 'solid-js';

type InputProps = {
    value: string;
    onInput?: JSX.EventHandler<HTMLInputElement, InputEvent>;
    onKeyDown?: JSX.EventHandler<HTMLInputElement, KeyboardEvent>;
    onFocus?: JSX.EventHandler<HTMLInputElement, FocusEvent>;
    onBlur?: JSX.EventHandler<HTMLInputElement, FocusEvent>;
    placeholder?: string;
    disabled?: boolean;
    readOnly?: boolean;
    ref?: HTMLInputElement | ((el: HTMLInputElement) => void);
    class?: string;
};

export function Input(props: InputProps) {
    return (
        <input
            ref={props.ref}
            type="text"
            value={props.value}
            onInput={props.onInput}
            onKeyDown={props.onKeyDown}
            onFocus={props.onFocus}
            onBlur={props.onBlur}
            placeholder={props.placeholder}
            disabled={props.disabled}
            readOnly={props.readOnly}
            class={`w-full px-3 py-2 rounded-md border border-border bg-bg text-fg
              placeholder:text-fg-muted focus:outline-none focus:ring-2 focus:ring-accent
              disabled:opacity-40 ${props.class ?? ''}`}
        />
    );
}
