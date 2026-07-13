import React, { createContext, useContext, useMemo, useRef } from 'react';
import { useStore, StoreApi } from 'zustand';
import { ChatAdapter, ChatCommonConfig, ComponentConfig } from '@/types';
import { createChatStore, IChatStore } from '@/store/createChatStore';
import { createHistoryStore, IHistoryStore } from '@/store/createHistoryStore';
import { createPreviewStore, IPreviewStore } from '@/store/createPreviewStore';
import { createDefaultAdapter } from '@/provider/adapters/defaultAdapter';
import { useThemeVars } from '@/hooks/useThemeVars';

interface ChatContextValue {
  config: ChatCommonConfig;
  adapter: ChatAdapter;
  chatStore: StoreApi<IChatStore>;
  historyStore: StoreApi<IHistoryStore>;
  previewStore: StoreApi<IPreviewStore>;
}

const ChatContext = createContext<ChatContextValue | null>(null);
const ComponentConfigContext = createContext<Record<string, any>>({});

export function ChatProvider({
  config,
  componentConfig = {},
  adapter: adapterProp,
  stores: externalStores,
  children,
}: {
  config: ChatCommonConfig;
  componentConfig?: ComponentConfig;
  adapter?: ChatAdapter;
  stores?: {
    chatStore: StoreApi<IChatStore>;
    historyStore: StoreApi<IHistoryStore>;
    previewStore: StoreApi<IPreviewStore>;
  };
  children: React.ReactNode;
}) {
  const storesRef = useRef<{
    chatStore: StoreApi<IChatStore>;
    historyStore: StoreApi<IHistoryStore>;
    previewStore: StoreApi<IPreviewStore>;
  }>(
    externalStores || {
      chatStore: createChatStore(),
      historyStore: createHistoryStore(),
      previewStore: createPreviewStore(),
    }
  );

  const adapter = useMemo(
    () => adapterProp || createDefaultAdapter(config),
    [adapterProp, config]
  );

  const value = useMemo<ChatContextValue>(
    () => ({
      config,
      adapter,
      chatStore: storesRef.current.chatStore,
      historyStore: storesRef.current.historyStore,
      previewStore: storesRef.current.previewStore,
    }),
    [config, adapter]
  );

  const themeVars = useThemeVars(config.themeConfig);

  return (
    <ChatContext.Provider value={value}>
      <ComponentConfigContext.Provider value={componentConfig as Record<string, any>}>
        <div className="p2p-chat-root" style={themeVars}>
          {children}
        </div>
      </ComponentConfigContext.Provider>
    </ChatContext.Provider>
  );
}

// ============================================================
// Hooks to access the scoped stores
// ============================================================

function useChatContext(): ChatContextValue {
  const ctx = useContext(ChatContext);
  if (!ctx) {
    throw new Error('useChatContext must be used within a <ChatProvider>');
  }
  return ctx;
}

export function useConfig(): ChatCommonConfig {
  return useChatContext().config;
}

export function useAdapter(): ChatAdapter {
  return useChatContext().adapter;
}

export function useChatStore(): StoreApi<IChatStore> {
  return useChatContext().chatStore;
}

export function useHistoryStore(): StoreApi<IHistoryStore> {
  return useChatContext().historyStore;
}

export function usePreviewStore(): StoreApi<IPreviewStore> {
  return useChatContext().previewStore;
}

export function useComponentConfig<T = Record<string, any>>(): T {
  return useContext(ComponentConfigContext) as T;
}

// Selector hooks for convenience
export function useChatStoreState<T>(selector: (state: IChatStore) => T): T {
  const store = useChatStore();
  return useStore(store, selector);
}

export function useHistoryStoreState<T>(selector: (state: IHistoryStore) => T): T {
  const store = useHistoryStore();
  return useStore(store, selector);
}

export function usePreviewStoreState<T>(selector: (state: IPreviewStore) => T): T {
  const store = usePreviewStore();
  return useStore(store, selector);
}
