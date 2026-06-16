import { render } from 'solid-js/web';
import { initTheme } from '@litools/design-tokens';
import App from './App';
import '@litools/ui/style.css';
import '@litools/design-tokens/index.css';
import './index.css';

initTheme();
render(() => <App />, document.getElementById('root')!);
