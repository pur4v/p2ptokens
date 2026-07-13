import React, { useState } from 'react';
import { Trash2 } from 'lucide-react';
import { IThread, ConcurrentThreadStatus } from '@/types';
import { ThreadStatusIcon } from '@/components/History/ThreadStatusIcon';

interface ThreadItemProps {
  thread: IThread;
  isActive: boolean;
  status?: ConcurrentThreadStatus;
  onClick: (threadId: string) => void;
  onDelete?: (threadId: string) => void;
}

export const ThreadItem: React.FC<ThreadItemProps> = ({
  thread,
  isActive,
  status,
  onClick,
  onDelete,
}) => {
  const [isHovered, setIsHovered] = useState(false);

  return (
    <button
      type="button"
      className="fc-w-full fc-flex fc-items-center fc-gap-2 fc-px-3 fc-py-2 fc-rounded-lg fc-transition-colors fc-text-left fc-border-0 fc-cursor-pointer"
      style={{
        backgroundColor: isActive ? 'color-mix(in srgb, var(--fc-primary) 10%, transparent)' : 'transparent',
        color: isActive ? 'var(--fc-primary)' : 'var(--fc-text)',
      }}
      onClick={() => onClick(thread.id)}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
    >
      <ThreadStatusIcon status={status} size={14} />
      <span className="fc-flex-1 fc-text-sm fc-truncate">
        {thread.title || 'Untitled'}
      </span>
      {isHovered && onDelete && (
        <button
          type="button"
          className="fc-p-1 fc-rounded fc-transition-colors fc-bg-transparent fc-border-0 fc-cursor-pointer"
          style={{ color: 'var(--fc-text-secondary)' }}
          onClick={(e) => {
            e.stopPropagation();
            onDelete(thread.id);
          }}
          title="Delete thread"
        >
          <Trash2 size={14} />
        </button>
      )}
    </button>
  );
};
