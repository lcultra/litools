import { ToggleGroup } from '@kobalte/core/toggle-group';
import { createEffect, createSignal, onMount } from 'solid-js';

interface SegmentedOption<T extends string> {
  value: T;
  label: string;
}

interface SegmentedControlProps<T extends string> {
  items: readonly SegmentedOption<T>[];
  value: T;
  onChange: (value: T) => void;
  disabled?: boolean;
}

export function SegmentedControl<T extends string>(props: SegmentedControlProps<T>) {
  let groupRef!: HTMLDivElement;
  const itemRefs = new Map<string, HTMLButtonElement>();
  const [indicatorStyle, setIndicatorStyle] = createSignal<{
    left: string;
    width: string;
  }>();
  const [ready, setReady] = createSignal(false);

  const measure = () => {
    const groupEl = groupRef;
    const itemEl = itemRefs.get(props.value);
    if (!groupEl || !itemEl) return;
    const groupRect = groupEl.getBoundingClientRect();
    const itemRect = itemEl.getBoundingClientRect();
    setIndicatorStyle({
      left: `${itemRect.left - groupRect.left}px`,
      width: `${itemRect.width}px`,
    });
    setReady(true);
  };

  onMount(() => requestAnimationFrame(measure));

  createEffect(() => {
    // 追踪 value 变化，等 DOM 更新后重新测量
    props.value;
    requestAnimationFrame(measure);
  });

  return (
    <ToggleGroup
      ref={groupRef}
      value={props.value}
      onChange={(v) => {
        if (v) props.onChange(v as T);
      }}
      disabled={props.disabled}
      class="inline-flex relative rounded-lg border border-border bg-surface p-0.5"
    >
      {props.items.map((item) => (
        <ToggleGroup.Item
          ref={(el) => {
            if (el) itemRefs.set(item.value, el);
          }}
          value={item.value}
          class="relative z-10 px-3 py-1.5 h-8 text-sm font-medium rounded-md transition-colors whitespace-nowrap text-text-muted cursor-pointer hover:text-text data-pressed:text-text data-disabled:opacity-40 data-disabled:cursor-not-allowed"
        >
          {item.label}
        </ToggleGroup.Item>
      ))}
      {/* 滑动指示器放在最后，避免影响 flex 布局 */}
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
    </ToggleGroup>
  );
}
