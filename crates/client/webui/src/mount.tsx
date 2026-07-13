import React from 'react';
import { createRoot, Root } from 'react-dom/client';
import { StoreApi } from 'zustand';
import { ChatProvider } from '@/provider/ChatProvider';
import { componentRegistry } from '@/components/registry';
import { ChatCommonConfig, ChatAdapter, ComponentConfig } from '@/types';
import { createChatStore, IChatStore } from '@/store/createChatStore';
import { createHistoryStore, IHistoryStore } from '@/store/createHistoryStore';
import { createPreviewStore, IPreviewStore } from '@/store/createPreviewStore';

export interface SharedStores {
  chatStore: StoreApi<IChatStore>;
  historyStore: StoreApi<IHistoryStore>;
  previewStore: StoreApi<IPreviewStore>;
}

const roots = new Map<string, Root>();
const sharedStores = new WeakMap<ChatCommonConfig, SharedStores>();

function getOrCreateStores(config: ChatCommonConfig): SharedStores {
  let stores = sharedStores.get(config);
  if (!stores) {
    stores = {
      chatStore: createChatStore(),
      historyStore: createHistoryStore(),
      previewStore: createPreviewStore(),
    };
    sharedStores.set(config, stores);
  }
  return stores;
}

export function mountComponent(
  name: string,
  selector: string,
  commonConfig: ChatCommonConfig,
  componentConfig?: ComponentConfig,
  adapter?: ChatAdapter
): { unmount: () => void } {
  const container = document.querySelector(selector);
  if (!container) throw new Error(`Element not found: ${selector}`);

  const Component = componentRegistry[name];
  if (!Component) throw new Error(`Unknown component: ${name}. Available: ${Object.keys(componentRegistry).join(', ')}`);

  const stores = getOrCreateStores(commonConfig);

  const root = createRoot(container);
  root.render(
    <ChatProvider
      config={commonConfig}
      componentConfig={componentConfig || {}}
      adapter={adapter}
      stores={stores}
    >
      <Component />
    </ChatProvider>
  );

  roots.set(selector, root);
  return { unmount: () => unmountComponent(selector) };
}

export function unmountComponent(selector: string): void {
  const root = roots.get(selector);
  if (root) {
    root.unmount();
    roots.delete(selector);
  }
}
