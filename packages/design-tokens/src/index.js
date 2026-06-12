import { getTheme, onThemeChange } from './theme.js';

/**
 * 初始化主题：设置初始主题并监听主题变化。
 * @returns {() => void} 取消监听函数
 */
export function initTheme() {
  document.documentElement.dataset.theme = getTheme();

  return onThemeChange((t) => {
    document.documentElement.dataset.theme = t;
  });
}

export { getTheme, onThemeChange };
