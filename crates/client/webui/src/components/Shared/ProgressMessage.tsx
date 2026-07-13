import React, { useEffect, useState } from 'react';

interface ProgressMessageWithAnimationProps {
  progressMessage: string;
  isStreaming: boolean;
  progressMessageClickHandler: () => void;
}

export const ProgressMessageWithAnimation: React.FC<ProgressMessageWithAnimationProps> = ({
  progressMessage,
  isStreaming,
  progressMessageClickHandler,
}) => {
  const [dotCount, setDotCount] = useState(1);

  useEffect(() => {
    if (!isStreaming) return;
    const id = window.setInterval(() => {
      setDotCount((prev) => (prev % 3) + 1);
    }, 400);
    return () => window.clearInterval(id);
  }, [isStreaming]);

  const dots = isStreaming ? '.'.repeat(dotCount) : '...';

  return (
    <button
      type="button"
      className="fc-text-sm fc-tracking-wide fc-bg-clip-text fc-text-transparent fc-bg-[length:200%_100%] fc-animate-pulse fc-w-fit fc-bg-transparent fc-border-0 fc-cursor-pointer fc-p-0 fc-text-left"
      style={{ backgroundImage: `linear-gradient(to right, var(--fc-primary), color-mix(in srgb, var(--fc-primary) 70%, black), var(--fc-primary))` }}
      onClick={progressMessageClickHandler}
    >
      {progressMessage}
      {dots}
    </button>
  );
};
