import { For, Show } from 'solid-js';
import type { MatchRange } from '../../bridge/types';

type HighlightedTextProps = {
    class?: string;
    ranges?: MatchRange[];
    text: string;
};

type TextSegment = {
    highlighted: boolean;
    text: string;
};

export function HighlightedText(props: HighlightedTextProps) {
    const segments = () => textSegments(props.text, props.ranges ?? []);

    return (
        <span class={props.class}>
            <For each={segments()}>
                {(segment) => (
                    <Show when={segment.highlighted} fallback={segment.text}>
                        <span class="font-semibold text-danger">{segment.text}</span>
                    </Show>
                )}
            </For>
        </span>
    );
}

function textSegments(text: string, ranges: MatchRange[]): TextSegment[] {
    const chars = Array.from(text);
    const normalizedRanges = normalizeRanges(ranges, chars.length);
    if (!normalizedRanges.length) {
        return [{ highlighted: false, text }];
    }

    const segments: TextSegment[] = [];
    let cursor = 0;

    for (const range of normalizedRanges) {
        if (cursor < range.start) {
            segments.push({ highlighted: false, text: chars.slice(cursor, range.start).join('') });
        }

        segments.push({ highlighted: true, text: chars.slice(range.start, range.end).join('') });
        cursor = range.end;
    }

    if (cursor < chars.length) {
        segments.push({ highlighted: false, text: chars.slice(cursor).join('') });
    }

    return segments;
}

function normalizeRanges(ranges: MatchRange[], textLength: number) {
    const clamped = ranges
        .map((range) => ({
            start: Math.max(0, Math.min(textLength, range.start)),
            end: Math.max(0, Math.min(textLength, range.end)),
        }))
        .filter((range) => range.start < range.end)
        .sort((left, right) => left.start - right.start || left.end - right.end);

    const merged: MatchRange[] = [];
    for (const range of clamped) {
        const previous = merged[merged.length - 1];
        if (previous && range.start <= previous.end) {
            previous.end = Math.max(previous.end, range.end);
        } else {
            merged.push({ ...range });
        }
    }

    return merged;
}
