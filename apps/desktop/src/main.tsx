import { initTheme } from '@litools/design-tokens';
import { render } from 'solid-js/web';
import { App } from './App';
import './styles.css';

initTheme();

const root = document.getElementById('root');

if (root) {
    render(() => <App />, root);
}
