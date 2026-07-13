import React, { ReactNode, useEffect, useRef } from 'react';

interface AutoScrollContainerProps {
  messages: any[];
  children: ReactNode;
  className?: string;
  activeThreadId?: string | null;
}

export const AutoScrollContainer: React.FC<AutoScrollContainerProps> = ({
  messages,
  children,
  className = '',
  activeThreadId,
}) => {
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const prevThreadId = useRef<string | null>(null);

  useEffect(() => {
    if (activeThreadId && activeThreadId !== prevThreadId.current && messages.length > 0) {
      setTimeout(() => {
        messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
      }, 100);
      prevThreadId.current = activeThreadId;
    }
  }, [activeThreadId, messages.length]);

  useEffect(() => {
    if (messages.length > 0) {
      messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
    }
  }, [messages.length]);

  return (
    <div className={className}>
      {children}
      <div ref={messagesEndRef} />
    </div>
  );
};
