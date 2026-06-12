import { Tooltip as KobalteTooltip } from '@kobalte/core/tooltip';
import type { ParentComponent } from 'solid-js';

type TooltipPlacement =
  | 'top'
  | 'bottom'
  | 'left'
  | 'right'
  | 'top-start'
  | 'top-end'
  | 'bottom-start'
  | 'bottom-end'
  | 'left-start'
  | 'left-end'
  | 'right-start'
  | 'right-end';

interface TooltipProps {
  content: string;
  placement?: TooltipPlacement;
}

export const Tooltip: ParentComponent<TooltipProps> = (props) => {
  return (
    <KobalteTooltip
      openDelay={400}
      closeDelay={100}
      placement={props.placement ?? 'top'}
    >
      <KobalteTooltip.Trigger as="span" class="inline-flex cursor-pointer">
        {props.children}
      </KobalteTooltip.Trigger>
      <KobalteTooltip.Portal>
        <KobalteTooltip.Content class="z-50 px-3 py-1.5 text-xs font-medium bg-surface-elevated text-text rounded-md shadow-lg border border-border data-expanded:animate-in data-closed:animate-out">
          <KobalteTooltip.Arrow />
          {props.content}
        </KobalteTooltip.Content>
      </KobalteTooltip.Portal>
    </KobalteTooltip>
  );
};
