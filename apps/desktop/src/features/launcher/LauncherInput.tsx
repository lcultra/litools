import { useNavigate } from '@solidjs/router';
import { createEffect, createSignal, onCleanup } from 'solid-js';
import { openPluginView, startWindowDragging } from '../../bridge/commands';
import { generatePluginPath } from '../../shared/routes';

const SETTINGS_PLUGIN_ID = 'dev.litools.settings';
const SETTINGS_COMMAND_ID = 'settings';

const SEARCH_INPUT_HEIGHT = 68;
const SEARCH_INPUT_LEFT_PADDING = 16;
const SEARCH_INPUT_RIGHT_ACTION_WIDTH = 60;
const SEARCH_INPUT_ACTION_DRAG_THRESHOLD = 4;
const SEARCH_INPUT_PLACEHOLDER = '搜索应用、命令、文件、插件...';

type LauncherInputProps = {
    inputRef: (element: HTMLInputElement) => void;
    onInput: (value: string) => void;
    onInputBlur: () => void;
    onKeyDown: (event: KeyboardEvent) => void;
    onSubmit: (event: SubmitEvent) => void;
    query: string;
};

export function LauncherInput(props: LauncherInputProps) {
    const navigate = useNavigate();
    let rootElement: HTMLFormElement | undefined;
    let measureElement: HTMLSpanElement | undefined;
    let actionPointerStart: { x: number; y: number } | undefined;
    let actionDragStarted = false;
    const [inputContentWidth, setInputContentWidth] = createSignal<number | undefined>();

    function measureInputWidth() {
        if (!rootElement) {
            return;
        }

        const rootWidth = rootElement.getBoundingClientRect().width;
        const maxContentWidth = Math.max(rootWidth - SEARCH_INPUT_RIGHT_ACTION_WIDTH, 0);

        if (!props.query) {
            setInputContentWidth(0);
            return;
        }

        const measuredWidth = measureElement?.getBoundingClientRect().width ?? 0;
        setInputContentWidth(Math.min(Math.ceil(measuredWidth) + SEARCH_INPUT_LEFT_PADDING, maxContentWidth));
    }

    function handleRootElement(element: HTMLFormElement) {
        rootElement = element;
        const observer = new ResizeObserver(measureInputWidth);
        observer.observe(element);
        measureInputWidth();

        onCleanup(() => observer.disconnect());
    }

    function handleDragPointerDown(event: PointerEvent) {
        if (event.button !== 0) {
            return;
        }

        void startWindowDragging();
    }

    function handleActionPointerDown(event: PointerEvent) {
        if (event.button !== 0) {
            return;
        }

        actionPointerStart = { x: event.clientX, y: event.clientY };
        actionDragStarted = false;
        (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
    }

    function handleActionPointerMove(event: PointerEvent) {
        if (!actionPointerStart || actionDragStarted) {
            return;
        }

        const deltaX = event.clientX - actionPointerStart.x;
        const deltaY = event.clientY - actionPointerStart.y;
        const distance = Math.hypot(deltaX, deltaY);

        if (distance < SEARCH_INPUT_ACTION_DRAG_THRESHOLD) {
            return;
        }

        actionDragStarted = true;
        void startWindowDragging();
    }

    function handleActionPointerUp() {
        actionPointerStart = undefined;
    }

    async function handleActionClick(event: MouseEvent) {
        if (actionDragStarted) {
            event.preventDefault();
            actionDragStarted = false;
            return;
        }

        const route = generatePluginPath(SETTINGS_PLUGIN_ID, SETTINGS_COMMAND_ID);
        try {
            const info = await openPluginView(SETTINGS_PLUGIN_ID, SETTINGS_COMMAND_ID);
            if (info.placement === 'docked') {
                navigate(route, { state: { runtimeId: info.runtimeId } });
            }
        } catch {
            // 静默失败，避免拖拽时的意外触发影响体验
        }
    }

    createEffect(() => {
        props.query;
        queueMicrotask(measureInputWidth);
    });

    return (
        <form ref={handleRootElement} class="relative flex h-[68px] border-b border-border" onSubmit={props.onSubmit}>
            <input
                ref={props.inputRef}
                aria-autocomplete="none"
                autocapitalize="off"
                autocomplete="off"
                autocorrect="off"
                autofocus
                class="h-full w-full border-0 bg-transparent py-2 pl-4 pr-[60px] text-2xl leading-[2rem] text-fg outline-none placeholder:text-muted"
                data-launcher-no-drag
                id="launcher-search"
                inputmode="search"
                name="launcher-search"
                on:blur={props.onInputBlur}
                on:keydown={props.onKeyDown}
                onInput={(event) => props.onInput(event.currentTarget.value)}
                placeholder={SEARCH_INPUT_PLACEHOLDER}
                spellcheck={false}
                value={props.query}
            />
            <div
                aria-hidden="true"
                class="absolute bottom-0 top-0"
                onPointerDown={handleDragPointerDown}
                style={{ left: props.query ? `${inputContentWidth() ?? 0}px` : '0', right: `${SEARCH_INPUT_RIGHT_ACTION_WIDTH}px` }}
            />
            <button
                aria-label="打开设置"
                class="absolute right-3 top-1/2 grid size-9 -translate-y-1/2 cursor-pointer place-items-center rounded-full border border-border bg-surface-muted text-sm font-semibold text-muted transition-colors hover:border-accent/30 hover:bg-surface hover:text-fg focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/30"
                data-launcher-interactive
                onClick={handleActionClick}
                onPointerCancel={handleActionPointerUp}
                onPointerDown={handleActionPointerDown}
                onPointerMove={handleActionPointerMove}
                onPointerUp={handleActionPointerUp}
                tabindex={-1}
                type="button"
            >
                P
            </button>
            <span
                ref={measureElement}
                aria-hidden="true"
                class="pointer-events-none invisible absolute whitespace-pre text-2xl leading-[2rem]"
                style={{ height: `${SEARCH_INPUT_HEIGHT}px` }}
            >
                {props.query}
            </span>
        </form>
    );
}
