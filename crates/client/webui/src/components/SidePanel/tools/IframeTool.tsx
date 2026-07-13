import React, { useEffect, useRef, useState } from 'react';
import clsx from 'clsx';
import { MousePointerClick } from 'lucide-react';

interface IframeToolProps {
  content: string;
  liveExpired?: boolean;
}

export const IframeTool = React.memo<IframeToolProps>(({ content, liveExpired = false }) => {
  const [isTakingControl, setIsTakingControl] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);
  const [isHovering, setIsHovering] = useState(false);

  useEffect(() => {
    const handleFullscreenChange = () => {
      if (!document.fullscreenElement && isTakingControl) {
        setIsTakingControl(false);
      }
    };
    document.addEventListener('fullscreenchange', handleFullscreenChange);
    return () => document.removeEventListener('fullscreenchange', handleFullscreenChange);
  }, [isTakingControl]);

  const handleTakeControl = () => {
    setIsTakingControl(true);
    containerRef.current?.requestFullscreen();
  };

  const handleReleaseControl = () => {
    setIsTakingControl(false);
    document.exitFullscreen();
  };

  if (liveExpired) {
    return (
      <div className="fc-relative fc-w-full fc-aspect-video fc-flex fc-items-center fc-justify-center fc-overflow-hidden" style={{ backgroundColor: 'var(--fc-surface-hover)' }}>
        <span className="fc-text-xs" style={{ color: 'var(--fc-error)' }}>Live session ended</span>
      </div>
    );
  }

  return (
    <>
      <div
        ref={containerRef}
        className="fc-relative fc-w-full fc-aspect-video fc-flex fc-items-center fc-justify-center fc-overflow-hidden"
        style={{ backgroundColor: 'var(--fc-surface-hover)' }}
      >
        <iframe
          title="Browser"
          src={content}
          className={clsx('fc-w-full fc-h-full', !isTakingControl && 'fc-pointer-events-none')}
        />
        {isTakingControl && (
          <div className="fc-absolute fc-bottom-4 fc-flex fc-justify-center fc-w-full">
            <button
              type="button"
              className="fc-px-4 fc-py-2 fc-rounded-full fc-border fc-cursor-pointer fc-text-sm"
              style={{ color: 'var(--fc-text)', backgroundColor: 'var(--fc-bg)', borderColor: 'var(--fc-border)' }}
              onClick={handleReleaseControl}
            >
              Exit Takeover
            </button>
          </div>
        )}
      </div>
      <div
        className="fc-flex fc-justify-end fc-gap-2 fc-absolute fc-bottom-4 fc-right-4"
        onMouseEnter={() => setIsHovering(true)}
        onMouseLeave={() => setIsHovering(false)}
      >
        <button
          type="button"
          className="fc-flex fc-gap-1 fc-items-center fc-cursor-pointer fc-px-3 fc-py-2 fc-rounded-full fc-border"
          style={{ color: 'var(--fc-text)', backgroundColor: 'var(--fc-bg)', borderColor: 'var(--fc-border)' }}
          onClick={handleTakeControl}
        >
          <MousePointerClick size={16} />
          {isHovering && <span className="fc-text-sm">Take control</span>}
        </button>
      </div>
    </>
  );
});

IframeTool.displayName = 'IframeTool';
