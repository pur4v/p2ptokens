import React, { useEffect, useRef, useState } from 'react';
import { Check, Copy } from 'lucide-react';

interface CopyContentProps {
  content: string;
  fixedPosition?: boolean;
  size?: number;
  isParentHovered?: boolean;
}

export const CopyContent: React.FC<CopyContentProps> = ({
  content,
  fixedPosition = true,
  size = 16,
  isParentHovered = false,
}) => {
  const [isCopied, setIsCopied] = useState(false);
  const [isVisible, setIsVisible] = useState(false);
  const hideTimeoutId = useRef<ReturnType<typeof setTimeout> | null>(null);
  const copiedTimeoutId = useRef<ReturnType<typeof setTimeout> | null>(null);

  const handleCopy = () => {
    try {
      navigator.clipboard.writeText(content);
      setIsCopied(true);
    } catch (error) {
      console.warn('Failed to copy content:', error);
    }
  };

  useEffect(() => {
    if (isParentHovered) {
      if (hideTimeoutId.current) {
        clearTimeout(hideTimeoutId.current);
        hideTimeoutId.current = null;
      }
      setIsVisible(true);
    } else {
      hideTimeoutId.current = setTimeout(() => setIsVisible(false), 2000);
    }
    return () => {
      if (hideTimeoutId.current) clearTimeout(hideTimeoutId.current);
    };
  }, [isParentHovered]);

  useEffect(() => {
    if (isCopied) {
      copiedTimeoutId.current = setTimeout(() => setIsCopied(false), 1000);
    }
    return () => {
      if (copiedTimeoutId.current) clearTimeout(copiedTimeoutId.current);
    };
  }, [isCopied]);

  return (
    <div
      className={`${fixedPosition ? 'fc-absolute fc-top-full fc-right-0 fc-pt-1' : ''} fc-flex fc-items-center fc-justify-center fc-transition-opacity ${isVisible ? 'fc-opacity-100' : 'fc-opacity-0 fc-pointer-events-none'}`}
      style={{ transitionDuration: '300ms' }}
    >
      <button
        title="Copy"
        type="button"
        className="fc-rounded-lg fc-transition-colors fc-duration-300 fc-bg-transparent fc-border-0 fc-cursor-pointer fc-p-1"
        onClick={(e) => {
          e.stopPropagation();
          handleCopy();
        }}
      >
        {isCopied ? (
          <Check size={size} style={{ color: 'var(--fc-success)' }} />
        ) : (
          <Copy size={size} style={{ color: 'var(--fc-text-secondary)' }} />
        )}
      </button>
    </div>
  );
};
