import { HashRouter, Route } from '@solidjs/router';
import { render } from 'solid-js/web';
import { AppLayout } from './App';
import { Launcher } from './features/launcher/Launcher';
import { WorkspacePage } from './features/workspace/WorkspacePage';
import { PLUGIN_ROUTE_PATTERN } from './shared/routes';
import './styles.css';

const root = document.getElementById('root');

if (root) {
    render(
        () => (
            <HashRouter>
                <Route
                    path="/"
                    component={() => (
                        <AppLayout>
                            <Launcher />
                        </AppLayout>
                    )}
                />
                <Route
                    path={PLUGIN_ROUTE_PATTERN}
                    component={() => (
                        <AppLayout>
                            <WorkspacePage />
                        </AppLayout>
                    )}
                />
            </HashRouter>
        ),
        root,
    );
}
