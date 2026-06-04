import { PageHeader } from '../../components/PageHeader';
import { Panel } from '../../components/Panel';

export function PluginManagerPage() {
    return (
        <Panel>
            <PageHeader description="管理插件和扩展能力。" title="插件" />
            <p class="m-0 mt-6 rounded-xl bg-surface-muted px-4 py-3 text-sm text-muted">插件管理功能将在后续版本开放。</p>
        </Panel>
    );
}
