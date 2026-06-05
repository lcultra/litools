import { move } from '@dnd-kit/helpers';
import { DragDropProvider } from '@dnd-kit/solid';
import { useSortable } from '@dnd-kit/solid/sortable';
import { createSignal, For } from 'solid-js';
import { PageHeader } from '../../components/PageHeader';
import { Panel } from '../../components/Panel';

const dragDemoItems = createRange(20);

type DragEndEvent = Parameters<typeof move>[1];

type SortableDemoItemProps = {
    id: number;
    index: number;
};

function SortableDemoItem(props: SortableDemoItemProps) {
    const sortable = useSortable({
        get id() {
            return props.id;
        },
        get index() {
            return props.index;
        },
    });

    return (
        <div
            ref={sortable.ref}
            class="grid h-full cursor-grab place-items-center justify-center rounded-2xl border border-border bg-surface text-lg font-semibold text-fg shadow-sm transition-[opacity,box-shadow,border-color] active:cursor-grabbing"
            classList={{
                'opacity-45 shadow-xl': sortable.isDragging(),
                'border-accent/70': sortable.isDropTarget(),
            }}
        >
            {props.id}
        </div>
    );
}

function createRange(length: number) {
    return Array.from({ length }, (_, index) => index + 1);
}

export function PluginManagerPage() {
    const [items, setItems] = createSignal<number[]>(dragDemoItems);

    function handleDragEnd(event: DragEndEvent) {
        setItems((current) => move(current, event));
    }

    return (
        <Panel>
            <PageHeader description="管理插件和扩展能力。" title="插件" />
            <div class="mt-6 grid gap-4 rounded-xl bg-surface-muted p-4">
                <p class="m-0 text-sm text-muted">拖拽下面的方块测试 dnd-kit sortable 网格排序能力。</p>
                <DragDropProvider onDragEnd={handleDragEnd}>
                    <div class="mx-auto grid max-w-[900px] grid-cols-[repeat(auto-fill,150px)] auto-rows-[150px] grid-flow-dense justify-center gap-[18px] px-[30px]">
                        <For each={items()}>{(item, index) => <SortableDemoItem id={item} index={index()} />}</For>
                    </div>
                </DragDropProvider>
            </div>
        </Panel>
    );
}
