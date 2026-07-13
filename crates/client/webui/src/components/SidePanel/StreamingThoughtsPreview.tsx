import React from 'react';
import { usePreviewStoreState, useConfig } from '@/provider/ChatProvider';
import { ToolTop } from '@/components/SidePanel/ToolTop';
import { ThoughtStreamComponent } from '@/components/Messages/ThoughtStream';

export const StreamingThoughtsPreview: React.FC = () => {
  const AgentPreviewObject = usePreviewStoreState((s) => s.AgentPreviewObject);
  const setAgentPreviewObject = usePreviewStoreState((s) => s.setAgentPreviewObject);
  const config = useConfig();

  const noThoughtsText = config.textConfig?.noThoughtsText || 'No thoughts available';
  const thoughts = AgentPreviewObject?.streamingThoughts ?? [];

  return (
    <div className="fc-h-full fc-w-full fc-flex fc-flex-col fc-rounded-xl fc-overflow-hidden fc-border" style={{ backgroundColor: 'var(--fc-bg)', borderColor: 'var(--fc-border)' }}>
      <ToolTop title="Thoughts" onClose={() => setAgentPreviewObject(null)} />
      <div className="fc-flex-1 fc-overflow-y-auto fc-p-4">
        {thoughts.length > 0 ? (
          <ThoughtStreamComponent streamingThoughts={thoughts} />
        ) : (
          <p className="fc-text-sm" style={{ color: 'var(--fc-text-secondary)' }}>{noThoughtsText}</p>
        )}
      </div>
    </div>
  );
};
