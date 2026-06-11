/** 阻止默认行为，允许事件冒泡 */
export function preventDefault(event: Event): void {
    event.preventDefault();
}

/** 阻止默认行为并阻止事件冒泡 */
export function stopEvent(event: Event): void {
    event.preventDefault();
    event.stopPropagation();
}
