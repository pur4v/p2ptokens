import React, { useState } from 'react';
import clsx from 'clsx';
import { ChevronRight } from 'lucide-react';
import { CodeViewerTool } from './CodeViewerTool';

interface RunCodeToolProps {
  content: string;
  outputContent: string;
}

export const RunCodeTool: React.FC<RunCodeToolProps> = ({ content, outputContent }) => {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <div className="fc-h-full fc-w-full fc-relative fc-flex fc-flex-col">
      <div className="fc-flex-1 fc-min-h-0 fc-pb-8">
        <CodeViewerTool content={content} language="python" />
      </div>
      <div className="fc-absolute fc-bottom-0 fc-w-full fc-z-10" style={{ backgroundColor: 'var(--fc-bg)' }}>
        <div className="fc-flex fc-flex-col fc-justify-center fc-px-2 fc-border-t" style={{ borderColor: 'var(--fc-border)' }}>
          <button
            type="button"
            className="fc-flex fc-items-center fc-gap-1 fc-cursor-pointer fc-bg-transparent fc-border-0 fc-p-0"
            onClick={() => setIsOpen(!isOpen)}
          >
            <ChevronRight
              size={14}
              strokeWidth={2}
              className={clsx(
                'fc-transition-transform fc-duration-300',
                isOpen && 'fc-rotate-90'
              )}
            />
            <span className="fc-text-sm fc-py-1" style={{ color: 'var(--fc-text)' }}>Output</span>
          </button>
          <div
            className={clsx(
              'fc-transition-[max-height] fc-duration-200 fc-ease-in-out fc-overflow-auto',
              isOpen ? 'fc-max-h-52' : 'fc-max-h-0'
            )}
          >
            <div className="fc-text-sm fc-p-2 fc-whitespace-pre-wrap" style={{ color: 'var(--fc-text)' }}>
              {outputContent}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
