import type { AppWatcherStatus, IconCacheSummary, IndexStatus, ReloadIndexSummary } from '../../bridge/types';

export function indexStatusSummary(status: IndexStatus) {
    if (status.running) {
        return status.pending ? '刷新中，已有待处理刷新' : '刷新中';
    }

    if (status.lastError) {
        return `失败：${status.lastError}`;
    }

    if (status.lastSummary?.success) {
        return '最近刷新成功';
    }

    return '空闲';
}

export function reloadSummary(summary: ReloadIndexSummary | null | undefined) {
    if (!summary) {
        return '暂无刷新记录';
    }

    return [
        summary.success ? '成功' : '失败',
        `触发：${summary.trigger}`,
        `应用：${summary.appsDiscovered}`,
        `移除：${summary.appsRemoved}`,
        `耗时：${summary.durationMs}ms`,
        summary.finishedAt,
    ].join('，');
}

export function watcherSummary(status: AppWatcherStatus) {
    if (status.enabled) {
        return `已启用（${status.status}）`;
    }

    return status.error ? `${status.status}：${status.error}` : status.status;
}

export function watchedPathsSummary(paths: string[]) {
    return paths.join('，') || '无';
}

export function iconCacheSummary(summary: IconCacheSummary) {
    const sizeMb = (summary.totalBytes / 1024 / 1024).toFixed(1);
    const parts = [`${summary.fileCount}/${summary.maxFiles} 个文件`, `${sizeMb}MB`];

    if (summary.lastPrunedAt) {
        parts.push(`最近清理 ${summary.lastPrunedFiles} 个`);
    }

    if (summary.error) {
        parts.push(`错误：${summary.error}`);
    }

    return parts.join('，');
}

export function windowBehaviorSummary(settings: { window: { hide_on_blur: boolean; close_to_tray: boolean; center_on_show: boolean } }) {
    return [
        settings.window.hide_on_blur ? '失焦时隐藏' : '失焦时保持显示',
        settings.window.close_to_tray ? '关闭到托盘' : '关闭即退出',
        settings.window.center_on_show ? '显示时居中' : '保持窗口位置',
    ].join('，');
}
