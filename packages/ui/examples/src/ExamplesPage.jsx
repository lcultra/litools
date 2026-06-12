import { createSignal } from 'solid-js';
import {
    Button,
    Input,
    Select,
    SegmentedControl,
    Switch,
    Badge,
    Card,
    PluginLayout,
    Tabs,
    Tooltip,
    ToastProvider,
    toast,
    ConfirmDialog
} from '@litools/plugin-ui';


const Section = (props) => (
    <section class="mb-8">
        <h2 class="text-lg font-semibold text-text mb-4">{props.name}</h2>
        <div class="flex flex-wrap items-start gap-4">{props.children}</div>
    </section>
);

export default function ExamplesPage() {
    const [inputVal, setInputVal] = createSignal('');
    const [selectVal, setSelectVal] = createSignal('');
    const [segVal, setSegVal] = createSignal('system');
    const [tabVal, setTabVal] = createSignal('a');
    const [switchVal, setSwitchVal] = createSignal(false);
    const [dialogOpen, setDialogOpen] = createSignal(false);

    return (
        <>
            <ToastProvider />
            <PluginLayout title="litools UI">
                <div>
                <Section name="Button">
                    <Button variant="primary">Primary</Button>
                    <Button variant="secondary">Secondary</Button>
                    <Button variant="danger">Danger</Button>
                    <Button variant="ghost">Ghost</Button>
                    <Button variant="primary" size="sm">Small</Button>
                    <Button variant="primary" disabled>Disabled</Button>
                </Section>

                <Section name="Input">
                    <Input label="名称" value={inputVal()} onChange={(e) => setInputVal(e.currentTarget.value)} placeholder="请输入" />
                    <Input label="带说明" description="辅助说明文字" value="" onChange={() => { }} />
                    <Input label="错误状态" error="不能为空" value="" onChange={() => { }} />
                </Section>

                <Section name="Select">
                    <Select label="框架" items={[{ value: 'solid', label: 'SolidJS' }, { value: 'react', label: 'React' }, { value: 'vue', label: 'Vue' }]} value={selectVal()} onChange={setSelectVal} />
                </Section>

                <Section name="SegmentedControl">
                    <SegmentedControl items={[{ value: 'system', label: '跟随系统' }, { value: 'light', label: '浅色' }, { value: 'dark', label: '深色' }]} value={segVal()} onChange={setSegVal} />
                </Section>

                <Section name="Switch">
                    <Switch checked={switchVal()} onChange={setSwitchVal} label="启用通知" />
                    <Switch checked={false} onChange={() => { }} disabled label="禁用状态" />
                </Section>

                <Section name="Badge">
                    <Badge>默认</Badge>
                    <Badge variant="success">成功</Badge>
                    <Badge variant="warning">警告</Badge>
                    <Badge variant="info">信息</Badge>
                    <Badge size="sm">小尺寸</Badge>
                </Section>

                <Section name="Card">
                    <Card>普通卡片</Card>
                    <Card onClick={() => console.log('clicked')}>可点击卡片</Card>
                </Section>

                <Section name="Tabs">
                    <Tabs items={[{ value: 'a', label: '标签 A' }, { value: 'b', label: '标签 B' }]} value={tabVal()} onChange={setTabVal}>
                        <Tabs.Panel value="a"><p class="text-text">内容 A</p></Tabs.Panel>
                        <Tabs.Panel value="b"><p class="text-text">内容 B</p></Tabs.Panel>
                    </Tabs>
                </Section>

                <Section name="Tooltip">
                    <Tooltip content="这是一个提示">
                        <Button variant="secondary">悬停查看</Button>
                    </Tooltip>
                </Section>

                <Section name="Toast">
                    <Button onClick={() => {
                         console.log('click');
                        toast.success('操作成功')}}>Success</Button>
                    <Button variant="danger" onClick={() => toast.error('操作失败')}>Error</Button>
                    <Button variant="secondary" onClick={() => toast.warning('请注意')}>Warning</Button>
                    <Button variant="ghost" onClick={() => toast.info('提示信息')}>Info</Button>
                </Section>

                <Section name="ConfirmDialog">
                    <Button variant="danger" onClick={() => setDialogOpen(true)}>删除</Button>
                    <ConfirmDialog
                        open={dialogOpen()}
                        onClose={() => setDialogOpen(false)}
                        onConfirm={() => { setDialogOpen(false); toast.success('已删除'); }}
                        title="确认删除"
                        description="此操作不可撤销"
                        variant="danger"
                    />
                </Section>
            </div>
        </PluginLayout>
        </>
    );
}
