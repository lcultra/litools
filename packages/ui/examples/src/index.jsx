import './index.css';
import { render } from 'solid-js/web';
import ExamplesPage from './ExamplesPage';
import '@litools/design-tokens/index.css';
import { initTheme } from '@litools/design-tokens';
import '@litools/ui/style.css';

initTheme();

render(
    () => <ExamplesPage />,
    document.getElementById('root')
);
