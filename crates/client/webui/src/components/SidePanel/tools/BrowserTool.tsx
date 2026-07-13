import React from 'react';

export const BrowserTool = React.memo(({ content }: { content: string }) => (
  <div className="fc-relative fc-w-full fc-aspect-video fc-flex fc-items-center fc-justify-center fc-overflow-hidden fc-h-full" style={{ backgroundColor: 'var(--fc-surface-hover)' }}>
    <img
      key={content}
      src={content}
      alt="Browser screenshot"
      loading="lazy"
      className="fc-object-contain fc-w-full fc-h-full fc-max-w-full fc-max-h-full"
    />
  </div>
));

BrowserTool.displayName = 'BrowserTool';
