import React from 'react';
import { ChatContainer } from '@/components/ChatContainer/ChatContainer';
import { ChatInput } from '@/components/Input/ChatInput';
import { HistorySidebar } from '@/components/History/HistorySidebar';
import { AssistantMessage } from '@/components/Messages/AssistantMessage';
import { UserMessage } from '@/components/Messages/UserMessage';
import { SidePanel } from '@/components/SidePanel/SidePanel';

export const componentRegistry: Record<string, React.ComponentType<any>> = {
  ChatContainer,
  ChatInput,
  History: HistorySidebar,
  AssistantMessage,
  UserMessage,
  SidePanel,
};
