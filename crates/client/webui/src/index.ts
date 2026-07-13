import '@/styles.css';
import { mountComponent, unmountComponent } from '@/mount';
import { ChatCommonConfig, ChatAdapter, ComponentConfig } from '@/types';

export type { ChatConfig, ChatCommonConfig, ChatAdapter } from '@/types';
export type {
  ChatThemeConfig,
  ChatTextConfig,
  ChatContainerConfig,
  ChatInputComponentConfig,
  HistoryComponentConfig,
  MessagesComponentConfig,
  ComponentConfig,
  IAssistantMessage,
  IUserMessage,
  IThread,
  IThreadState,
  ProductMode,
  ConcurrentThreadStatus,
  ProgressTrackerItem,
  UploadedImage,
  SelectedKB,
  MessageReaction,
  AgentPreviewObjectWithParams,
  IToolType,
  IToolHeaderMessage,
  SearchResult,
  BrowserToolProps,
  IframeToolProps,
  PlanToolProps,
} from '@/types';
export type { FileUploadItem, UseFileUploadOptions, UseFileUploadReturn } from '@/hooks/useFileUpload';

export { ChatProvider, useChatStore, useHistoryStore, usePreviewStore, useAdapter, useConfig, useComponentConfig } from '@/provider/ChatProvider';
export { createChatStore } from '@/store/createChatStore';
export { createHistoryStore } from '@/store/createHistoryStore';
export { createPreviewStore } from '@/store/createPreviewStore';
export { createDefaultAdapter } from '@/provider/adapters/defaultAdapter';

export { ChatContainer } from '@/components/ChatContainer/ChatContainer';
export { ChatInput } from '@/components/Input/ChatInput';
export { HistorySidebar } from '@/components/History/HistorySidebar';
export { AssistantMessage } from '@/components/Messages/AssistantMessage';
export { UserMessage } from '@/components/Messages/UserMessage';
export { SidePanel } from '@/components/SidePanel/SidePanel';

// Tool components (for custom compositions)
export { BrowserTool } from '@/components/SidePanel/tools/BrowserTool';
export { CodeViewerTool } from '@/components/SidePanel/tools/CodeViewerTool';
export { RunCodeTool } from '@/components/SidePanel/tools/RunCodeTool';
export { SearchEngineTool } from '@/components/SidePanel/tools/SearchEngineTool';
export { IframeTool } from '@/components/SidePanel/tools/IframeTool';
export { MarkdownTool } from '@/components/SidePanel/tools/MarkdownTool';

// Hooks
export { useFileUpload } from '@/hooks/useFileUpload';
export { useTheme } from '@/hooks/useTheme';
export { useThemeVars } from '@/hooks/useThemeVars';

export const render = (
  componentName: string,
  selector: string,
  commonConfig: ChatCommonConfig,
  componentConfigOrAdapter?: ComponentConfig | ChatAdapter,
  adapter?: ChatAdapter
) => {
  // Backward compat: if 4th arg has startStream, it's an adapter (old API)
  let resolvedComponentConfig: ComponentConfig = {};
  let resolvedAdapter = adapter;

  if (componentConfigOrAdapter) {
    if (typeof (componentConfigOrAdapter as ChatAdapter).startStream === 'function') {
      // Old API: render(name, sel, config, adapter)
      resolvedAdapter = componentConfigOrAdapter as ChatAdapter;
    } else {
      // New API: render(name, sel, config, componentConfig, adapter?)
      resolvedComponentConfig = componentConfigOrAdapter as ComponentConfig;
    }
  }

  return mountComponent(componentName, selector, commonConfig, resolvedComponentConfig, resolvedAdapter);
};

export const unmount = (selector: string) => {
  unmountComponent(selector);
};

export const version = '__VERSION__';
