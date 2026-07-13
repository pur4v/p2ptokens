import React, { useEffect, useState } from 'react';
import { ThumbsDown, ThumbsUp } from 'lucide-react';
import { useChatStore, useAdapter, useConfig } from '@/provider/ChatProvider';
import { MessageReaction } from '@/types';

interface LikeDislikeButtonsProps {
  size?: number;
  initialReaction?: MessageReaction;
}

export const LikeDislikeButtons: React.FC<LikeDislikeButtonsProps> = ({
  size = 16,
  initialReaction,
}) => {
  const [state, setState] = useState<'none' | 'liked' | 'disliked'>(() => {
    if (initialReaction?.type === 'liked') return 'liked';
    if (initialReaction?.type === 'disliked') return 'disliked';
    return 'none';
  });
  const chatStore = useChatStore();
  const adapter = useAdapter();
  const config = useConfig();
  const feedbackText = config.textConfig?.feedbackText || 'Thanks for the feedback!';
  const [lastPostedReaction, setLastPostedReaction] = useState<'none' | 'liked' | 'disliked'>(() => {
    if (initialReaction?.type) return initialReaction.type;
    return 'none';
  });
  const [showFeedbackChip, setShowFeedbackChip] = useState(false);

  useEffect(() => {
    if (initialReaction?.type) {
      setState(initialReaction.type);
      setLastPostedReaction(initialReaction.type);
    } else {
      setState('none');
      setLastPostedReaction('none');
    }
  }, [initialReaction]);

  useEffect(() => {
    if (showFeedbackChip) {
      const timer = setTimeout(() => setShowFeedbackChip(false), 3000);
      return () => clearTimeout(timer);
    }
  }, [showFeedbackChip]);

  const handleReaction = async (reaction: 'liked' | 'disliked') => {
    const { activeThreadId, getLastMessageId } = chatStore.getState();
    const messageId = getLastMessageId(activeThreadId);
    if (!activeThreadId || !messageId) return;
    if (reaction === lastPostedReaction) return;

    try {
      await adapter.submitReaction?.(activeThreadId, messageId, {
        type: reaction,
        dislike_reason_id: null,
        details: null,
      });
      setLastPostedReaction(reaction);
      if (reaction === 'liked') setShowFeedbackChip(true);
    } catch (error) {
      console.error('Error submitting reaction:', error);
    }
  };

  return (
    <div className="fc-flex fc-flex-col fc-gap-2">
      <div className="fc-flex fc-gap-2 fc-items-center">
        {state !== 'disliked' && (
          <button
            type="button"
            aria-label={state === 'liked' ? 'Liked' : 'Like'}
            className="fc-bg-transparent fc-border-0 fc-p-0 fc-cursor-pointer"
            style={{ color: 'var(--fc-text-secondary)' }}
            onClick={async () => {
              setState((p) => (p === 'liked' ? 'none' : 'liked'));
              if (state !== 'liked') await handleReaction('liked');
            }}
          >
            <ThumbsUp
              size={size}
              style={{ color: state === 'liked' ? 'var(--fc-text)' : 'var(--fc-text-secondary)' }}
              fill={state === 'liked' ? 'currentColor' : 'none'}
            />
          </button>
        )}
        {state !== 'liked' && (
          <button
            type="button"
            aria-label={state === 'disliked' ? 'Disliked' : 'Dislike'}
            className="fc-bg-transparent fc-border-0 fc-p-0 fc-cursor-pointer"
            style={{ color: 'var(--fc-text-secondary)' }}
            onClick={async () => {
              setState((p) => (p === 'disliked' ? 'none' : 'disliked'));
              if (state !== 'disliked') await handleReaction('disliked');
            }}
          >
            <ThumbsDown
              size={size}
              style={{ color: state === 'disliked' ? 'var(--fc-text)' : 'var(--fc-text-secondary)' }}
              fill={state === 'disliked' ? 'currentColor' : 'none'}
            />
          </button>
        )}
      </div>
      {showFeedbackChip && (
        <div className="fc-flex fc-justify-center fc-mt-2 fc-absolute fc-bottom-[calc(100%+4px)] fc-right-0">
          <div className="fc-px-4 fc-py-2 fc-rounded-full fc-text-sm fc-font-medium fc-border fc-shadow-sm" style={{ backgroundColor: 'var(--fc-bg)', color: 'var(--fc-text)', borderColor: 'var(--fc-border)' }}>
            {feedbackText}
          </div>
        </div>
      )}
    </div>
  );
};
