import { createStore, StoreApi } from 'zustand';
import { AgentPreviewObjectWithParams, UploadedImage } from '@/types';

export interface IPreviewStore {
  AgentPreviewObject: AgentPreviewObjectWithParams;
  setAgentPreviewObject: (obj: AgentPreviewObjectWithParams) => void;
  threadChartObject: any;
  setThreadChartObject: (chartObject: any) => void;
  chatAttachmentPreview: UploadedImage[];
  setChatAttachmentPreview: (images: UploadedImage[]) => void;
  currentChatAttachmentIndex: number | null;
  setCurrentChatAttachmentIndex: (index: number | null) => void;
  goToPreviousAttachment: () => void;
  goToNextAttachment: () => void;
}

export function createPreviewStore(): StoreApi<IPreviewStore> {
  return createStore<IPreviewStore>((set, get) => ({
    AgentPreviewObject: null,
    setAgentPreviewObject: (obj) => set({ AgentPreviewObject: obj }),
    threadChartObject: [],
    setThreadChartObject: (chartObject) => set({ threadChartObject: chartObject }),
    currentChatAttachmentIndex: null,
    setCurrentChatAttachmentIndex: (index) => set({ currentChatAttachmentIndex: index }),
    chatAttachmentPreview: [],
    setChatAttachmentPreview: (images) => set({ chatAttachmentPreview: images }),

    goToPreviousAttachment: () => {
      const { currentChatAttachmentIndex } = get();
      if (currentChatAttachmentIndex !== null && currentChatAttachmentIndex > 0) {
        set({ currentChatAttachmentIndex: currentChatAttachmentIndex - 1 });
      }
    },

    goToNextAttachment: () => {
      const { currentChatAttachmentIndex, chatAttachmentPreview } = get();
      if (
        currentChatAttachmentIndex !== null &&
        currentChatAttachmentIndex < chatAttachmentPreview.length - 1
      ) {
        set({ currentChatAttachmentIndex: currentChatAttachmentIndex + 1 });
      }
    },
  }));
}
