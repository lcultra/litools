import type { ParentComponent } from 'solid-js';

interface PluginLayoutProps {
  title?: string;
}

export const PluginLayout: ParentComponent<PluginLayoutProps> = (props) => {
  return (
    <main class="h-screen flex flex-col bg-surface text-text">
      {props.title && (
        <header class="shrink-0 px-4 py-3 border-b border-border">
          <h1 class="text-lg font-semibold">{props.title}</h1>
        </header>
      )}
      <div class="flex-1 overflow-auto px-4 py-4">{props.children}</div>
    </main>
  );
};
