import { TextField } from '@kobalte/core/text-field';
import type { JSX } from 'solid-js';

interface InputProps {
  label?: string;
  description?: string;
  error?: string;
  value: string;
  onChange?: JSX.EventHandler<HTMLInputElement, InputEvent>;
  placeholder?: string;
  disabled?: boolean;
  readOnly?: boolean;
  onKeyDown?: JSX.EventHandler<HTMLInputElement, KeyboardEvent>;
  onFocus?: JSX.EventHandler<HTMLInputElement, FocusEvent>;
  onBlur?: JSX.EventHandler<HTMLInputElement, FocusEvent>;
  ref?: (el: HTMLInputElement) => void;
  class?: string;
}

export function Input(props: InputProps) {
  return (
    <TextField
      value={props.value}
      onChange={(v) => {
        // Kobalte onChange 给的是 string，适配原 API 的 EventHandler
        if (props.onChange) {
          const fakeEvent = { currentTarget: { value: v } } as any;
          props.onChange(fakeEvent);
        }
      }}
      validationState={props.error ? 'invalid' : 'valid'}
      disabled={props.disabled}
      readOnly={props.readOnly}
    >
      {props.label && (
        <TextField.Label class="block text-sm font-medium text-text mb-1">
          {props.label}
        </TextField.Label>
      )}
      <TextField.Input
        placeholder={props.placeholder}
        onKeyDown={props.onKeyDown}
        onFocus={props.onFocus}
        onBlur={props.onBlur}
        ref={props.ref}
        class={`w-full px-3 py-1.5 h-8 rounded-md border border-border bg-surface-card text-text text-sm placeholder:text-text-muted focus:outline-none focus:ring-1 focus:ring-ring data-invalid:border-danger data-invalid:focus:ring-danger disabled:opacity-40 ${props.class ?? ''}`}
      />
      {props.description && !props.error && (
        <TextField.Description class="block text-xs text-text-muted mt-1">
          {props.description}
        </TextField.Description>
      )}
      {props.error && (
        <TextField.ErrorMessage class="block text-xs text-danger mt-1">
          {props.error}
        </TextField.ErrorMessage>
      )}
    </TextField>
  );
}
