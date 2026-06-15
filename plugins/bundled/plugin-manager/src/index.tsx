import { render } from 'solid-js/web';
import '@litools/design-tokens';
import App from './App';
import '@litools/ui/style.css';
import './index.css';

render(() => <App />, document.getElementById('root')!);
