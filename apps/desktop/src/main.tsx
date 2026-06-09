import { HashRouter, Route } from '@solidjs/router';
import { render } from 'solid-js/web';
import { AppShell } from './App';
import { Launcher } from './features/launcher/Launcher';
import { WorkspacePage } from './features/workspace/WorkspacePage';
import './styles.css';

const root = document.getElementById('root');

if (root) {
    render(
        () => (
            <HashRouter>
                <Route
                    path="/"
                    component={() => (
                        <AppShell>
                            <Launcher />
                        </AppShell>
                    )}
                />
                <Route
                    path="/plugin/:pluginId/:commandId"
                    component={() => (
                        <AppShell>
                            <WorkspacePage />
                        </AppShell>
                    )}
                />
            </HashRouter>
        ),
        root,
    );
}
