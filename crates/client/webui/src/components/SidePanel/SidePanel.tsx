import React from 'react';
import { AnimatePresence, motion } from 'framer-motion';
import { usePreviewStoreState, useConfig } from '@/provider/ChatProvider';
import { ComputerToolPreview } from '@/components/SidePanel/ComputerToolPreview';
import { StreamingThoughtsPreview } from '@/components/SidePanel/StreamingThoughtsPreview';

interface SidePanelProps {
  className?: string;
}

export const SidePanel: React.FC<SidePanelProps> = ({ className = '' }) => {
  const AgentPreviewObject = usePreviewStoreState((s) => s.AgentPreviewObject);
  const setAgentPreviewObject = usePreviewStoreState((s) => s.setAgentPreviewObject);
  const config = useConfig();

  let renderPreviewComponent: React.ReactNode = null;

  if (AgentPreviewObject) {
    switch (AgentPreviewObject.type) {
      case 'computer':
        renderPreviewComponent = <ComputerToolPreview />;
        break;
      case 'streaming_toughts_preview':
        renderPreviewComponent = <StreamingThoughtsPreview />;
        break;
      default: {
        // Check for custom renderer in config
        const CustomRenderer = config.toolRenderers?.[AgentPreviewObject.type ?? ''];
        if (CustomRenderer) {
          renderPreviewComponent = <CustomRenderer previewObject={AgentPreviewObject} />;
        }
        break;
      }
    }
  }

  return (
    <AnimatePresence>
      {renderPreviewComponent && (
        <motion.div
          className={`fc-fixed fc-top-0 fc-left-0 fc-w-full fc-h-full fc-z-50
            md:fc-relative md:fc-top-auto md:fc-left-auto md:fc-w-[400px] md:fc-h-full md:fc-z-auto md:fc-ml-4
            lg:fc-flex-1 lg:fc-w-auto ${className}`}
          initial={{ opacity: 0, scale: 0.95 }}
          animate={{ opacity: 1, scale: 1 }}
          exit={{ opacity: 0, scale: 0.95 }}
          transition={{ duration: 0.3, ease: 'easeInOut' }}
        >
          {/* Mobile backdrop */}
          <div
            className="fc-absolute fc-inset-0 fc-bg-black/30 md:fc-hidden"
            onClick={() => setAgentPreviewObject(null)}
          />
          <div className="fc-relative fc-h-full fc-w-full md:fc-w-auto">
            {renderPreviewComponent}
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  );
};
