/**
 * 获取当前系统主题
 * @returns {'light' | 'dark'}
 */
export function getTheme() {
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

/**
 * 监听系统主题变化
 * @param {(theme: 'light' | 'dark') => void} cb
 * @returns {() => void} 取消监听函数
 */
export function onThemeChange(cb) {
  const mq = window.matchMedia('(prefers-color-scheme: dark)');
  const handler = (e) => cb(e.matches ? 'dark' : 'light');
  mq.addEventListener('change', handler);
  return () => mq.removeEventListener('change', handler);
}
