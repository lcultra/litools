import { render } from 'solid-js/web';
import '@litools/design-tokens/index.css';
import { initTheme } from '@litools/design-tokens';
import App from './App';
import './index.css';
import '@litools/plugin-ui/style.css';

initTheme();
render(() => <App />, document.getElementById('root')!);
