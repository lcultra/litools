import type { ParentComponent } from 'solid-js';

type BadgeVariant = 'default' | 'success' | 'warning' | 'info';
type BadgeSize = 'sm' | 'md';

interface BadgeProps {
  variant?: BadgeVariant;
  size?: BadgeSize;
}

const variantClass: Record<BadgeVariant, string> = {
  default: 'bg-surface-hover text-text-muted',
  success: 'bg-success/15 text-success',
  warning: 'bg-warning/15 text-warning',
  info: 'bg-primary/15 text-primary',
};

const sizeClass: Record<BadgeSize, string> = {
  sm: 'px-1.5 py-px text-[10px] leading-3',
  md: 'px-2 py-0.5 text-xs',
};

export const Badge: ParentComponent<BadgeProps> = (props) => {
  return (
    <span class={`inline-flex items-center rounded font-medium ${variantClass[props.variant ?? 'default']} ${sizeClass[props.size ?? 'md']}`}>
      {props.children}
    </span>
  );
};
