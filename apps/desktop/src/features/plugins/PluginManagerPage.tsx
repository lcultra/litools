import { Button } from '../../components/Button';
import { PageHeader } from '../../components/PageHeader';
import { PageState } from '../../components/PageState';

export function PluginManagerPage() {
    return (
        <>
            <PageHeader description="管理插件生命周期、配置和扩展能力。" title="插件中心" />
            <div class="mt-6 grid gap-4">
                <PageState description="插件系统尚未开放。后续这里会展示已安装插件、运行状态、配置入口和本地安装能力。" title="暂无可管理的插件" variant="empty">
                    <div class="mt-4 flex justify-center gap-2">
                        <Button disabled variant="secondary">
                            插件市场
                        </Button>
                        <Button disabled variant="secondary">
                            本地安装
                        </Button>
                    </div>
                </PageState>
            </div>
        </>
    );
}
