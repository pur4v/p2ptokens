import React from 'react';
import { Circle, CircleCheck, Eye, LoaderCircle, MessageSquare } from 'lucide-react';
import { ConcurrentThreadStatus } from '@/types';

interface ThreadStatusIconProps {
  status?: ConcurrentThreadStatus;
  size?: number;
}

export const ThreadStatusIcon: React.FC<ThreadStatusIconProps> = ({ status, size = 16 }) => {
  switch (status) {
    case ConcurrentThreadStatus.RUNNING:
      return <LoaderCircle size={size} className="fc-animate-spin" style={{ color: 'var(--fc-link)' }} />;
    case ConcurrentThreadStatus.COMPLETED:
      return <CircleCheck size={size} style={{ color: 'var(--fc-success)' }} />;
    case ConcurrentThreadStatus.PENDING_SEEN:
      return <Eye size={size} style={{ color: 'var(--fc-warning)' }} />;
    case ConcurrentThreadStatus.ACTIVE:
      return <Circle size={size} style={{ color: 'var(--fc-link)' }} />;
    default:
      return <MessageSquare size={size} style={{ color: 'var(--fc-text-secondary)' }} />;
  }
};
