import { getTheme, onThemeChange } from './theme.js';

/**
 * 初始化主题：设置初始主题并监听系统主题变化。
 * 调用方在应用入口调用一次即可。
 * @returns {() => void} 取消监听函数
 */
export function initTheme() {
  document.documentElement.dataset.theme = getTheme();

  return onThemeChange((t) => {
    document.documentElement.dataset.theme = t;
  });
}

export { getTheme, onThemeChange };
