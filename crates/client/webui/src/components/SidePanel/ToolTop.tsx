import React from 'react';
import { Minimize2 } from 'lucide-react';

interface ToolTopProps {
  title: string;
  onClose: () => void;
}

export const ToolTop: React.FC<ToolTopProps> = ({ title, onClose }) => (
  <div className="fc-flex fc-items-center fc-justify-between fc-p-4">
    <p className="fc-text-lg fc-font-semibold" style={{ color: 'var(--fc-text)' }}>{title}</p>
    <button
      type="button"
      className="fc-flex fc-items-center fc-bg-transparent fc-px-2 fc-justify-center fc-cursor-pointer fc-border-0"
      style={{ color: 'var(--fc-text-secondary)' }}
      onClick={onClose}
    >
      <Minimize2 size={18} />
    </button>
  </div>
);
