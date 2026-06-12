import { render } from 'solid-js/web';
import '@litools/design-tokens/index.css';
import { initTheme } from '@litools/design-tokens';
import App from './App';
import '@litools/plugin-ui/style.css';
import './index.css';

initTheme();
render(() => <App />, document.getElementById('root')!);
