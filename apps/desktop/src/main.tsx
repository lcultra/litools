import { HashRouter, Route } from '@solidjs/router';
import { render } from 'solid-js/web';
import { App } from './App';
import './styles.css';

const root = document.getElementById('root');

if (root) {
    render(
        () => (
            <HashRouter>
                <Route path="*" component={App} />
            </HashRouter>
        ),
        root,
    );
}
