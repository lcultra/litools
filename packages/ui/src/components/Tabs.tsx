import { Tabs as KobalteTabs } from '@kobalte/core/tabs';
import { createEffect, createSignal, onMount } from 'solid-js';
import type { JSX, ParentComponent } from 'solid-js';

interface TabItem {
  value: string;
  label: string;
}

interface TabsProps {
  items: TabItem[];
  value: string;
  onChange: (value: string) => void;
  disabled?: boolean;
  children?: JSX.Element;
}

interface TabsPanelProps {
  value: string;
}

const Panel: ParentComponent<TabsPanelProps> = (props) => {
  return (
    <KobalteTabs.Content value={props.value} class="mt-2 rounded-lg border border-border bg-surface-card p-3">
      {props.children}
    </KobalteTabs.Content>
  );
};

export function Tabs(props: TabsProps) {
  let listRef!: HTMLDivElement;
  const triggerRefs = new Map<string, HTMLButtonElement>();
  const [indicatorStyle, setIndicatorStyle] = createSignal<{
    left: string;
    width: string;
  }>();
  const [ready, setReady] = createSignal(false);

  const measure = () => {
    const listEl = listRef;
    const triggerEl = triggerRefs.get(props.value);
    if (!listEl || !triggerEl) return;
    const listRect = listEl.getBoundingClientRect();
    const triggerRect = triggerEl.getBoundingClientRect();
    setIndicatorStyle({
      left: `${triggerRect.left - listRect.left}px`,
      width: `${triggerRect.width}px`,
    });
    setReady(true);
  };

  onMount(() => requestAnimationFrame(measure));

  createEffect(() => {
    props.value;
    requestAnimationFrame(measure);
  });

  return (
    <KobalteTabs value={props.value} onChange={props.onChange} disabled={props.disabled}>
      <KobalteTabs.List
        ref={listRef}
        class="flex relative w-full items-center gap-0.5 rounded-lg border border-border bg-surface p-0.5"
      >
        {props.items.map((item) => (
          <KobalteTabs.Trigger
            ref={(el) => {
              if (el) triggerRefs.set(item.value, el);
            }}
            value={item.value}
            class="relative z-10 flex-1 px-3 py-1.5 h-8 text-sm font-medium rounded-md transition-colors whitespace-nowrap text-text-muted cursor-pointer hover:text-text data-selected:text-text data-disabled:opacity-40 data-disabled:cursor-not-allowed"
          >
            {item.label}
          </KobalteTabs.Trigger>
        ))}
        {/* 滑动指示器：手动测量定位，放在最后避免影响 flex 布局 */}
        <div
          class="absolute z-0 top-0.5 h-[calc(100%-4px)] rounded-md bg-surface-card shadow-sm transition-all duration-200 ease-out"
          classList={{
            'opacity-0': !ready(),
            'opacity-100': ready(),
          }}
          style={{
            left: indicatorStyle()?.left ?? '0px',
            width: indicatorStyle()?.width ?? '0px',
          }}
        />
      </KobalteTabs.List>
      {props.children}
    </KobalteTabs>
  );
}

Tabs.Panel = Panel;
