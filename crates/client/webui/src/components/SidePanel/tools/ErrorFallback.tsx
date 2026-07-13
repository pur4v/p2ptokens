import React from 'react';
import { AlertTriangle } from 'lucide-react';

export const ErrorFallback: React.FC = () => (
  <div className="fc-flex fc-flex-col fc-items-center fc-justify-center fc-h-full fc-gap-2 fc-p-4">
    <AlertTriangle size={24} style={{ color: 'var(--fc-text-secondary)' }} />
    <p className="fc-text-sm" style={{ color: 'var(--fc-text-secondary)' }}>Something went wrong rendering this tool</p>
  </div>
);
