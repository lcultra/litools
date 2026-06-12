import type { ParentComponent } from 'solid-js';

interface CardProps {
  onClick?: () => void;
  class?: string;
}

export const Card: ParentComponent<CardProps> = (props) => {
  const interactive = () => !!props.onClick;
  return (
    <div
      role={interactive() ? 'button' : undefined}
      tabindex={interactive() ? 0 : undefined}
      onClick={props.onClick}
      onKeyDown={interactive() ? (e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); props.onClick!(); } } : undefined}
      class={`rounded-lg border border-border bg-surface-card p-4 ${interactive() ? 'cursor-pointer hover:bg-surface-hover transition-colors' : ''} ${props.class ?? ''}`}
    >
      {props.children}
    </div>
  );
};
