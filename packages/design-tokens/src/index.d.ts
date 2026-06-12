export type ThemeVariant = 'light' | 'dark';

/** 初始化主题：设置初始主题并监听主题变化，返回取消监听函数 */
export function initTheme(): () => void;

/** 获取当前系统主题 */
export function getTheme(): ThemeVariant;

/** 监听系统主题变化，返回取消监听函数 */
export function onThemeChange(cb: (theme: ThemeVariant) => void): () => void;
