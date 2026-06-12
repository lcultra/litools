import { Button as KobalteButton } from '@kobalte/core/button';
import type { JSX, ParentComponent } from 'solid-js';

type ButtonVariant = 'primary' | 'secondary' | 'danger' | 'ghost';
type ButtonSize = 'sm' | 'md';

interface ButtonProps {
  variant?: ButtonVariant;
  size?: ButtonSize;
  disabled?: boolean;
  onClick?: JSX.EventHandler<HTMLButtonElement, MouseEvent>;
}

const BASE_CLASS =
  'inline-flex items-center justify-center border font-medium select-none transition-colors focus-visible:ring-1 focus-visible:ring-ring focus-visible:outline-none';

export const Button: ParentComponent<ButtonProps> = (props) => {
  const v = () => props.variant ?? 'secondary';
  const s = () => props.size ?? 'md';
  const d = () => props.disabled ?? false;

  return (
    <KobalteButton
      disabled={props.disabled}
      onClick={props.onClick}
      class={BASE_CLASS}
      classList={{
        // 基础色值
        'border-transparent bg-primary text-primary-fg': v() === 'primary',
        'border-border bg-surface-card text-text': v() === 'secondary',
        'border-transparent bg-danger text-danger-fg': v() === 'danger',
        'border-transparent text-text-muted': v() === 'ghost',
        // 交互态（disabled 时不渲染，从根本上杜绝 hover/active 生效）
        'cursor-pointer hover:bg-primary-hover active:bg-primary-active':
          v() === 'primary' && !d(),
        'cursor-pointer hover:bg-surface-hover':
          (v() === 'secondary' || v() === 'ghost') && !d(),
        'cursor-pointer hover:bg-danger-hover active:bg-danger-active':
          v() === 'danger' && !d(),
        // 禁用态
        'data-disabled:opacity-40 data-disabled:cursor-not-allowed': d(),
        // 尺寸
        'px-2 py-1 text-xs rounded': s() === 'sm',
        'px-3 py-1.5 text-sm rounded-md h-8': s() === 'md',
      }}
    >
      {props.children}
    </KobalteButton>
  );
};
