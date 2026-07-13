import React, { memo, useMemo, useState } from 'react';
import clsx from 'clsx';
import { Brain, ChevronRight, Sparkles } from 'lucide-react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { IAssistantChunks, IAssistantMessage } from '@/types';
import { CHAT_INPUT_THINKING_MESSAGE } from '@/utils/constants';
import { useMarkdownRenderers } from '@/components/Shared/MarkdownRenderer';
import { ProgressMessageWithAnimation } from '@/components/Shared/ProgressMessage';
import { FollowUpQuestions } from '@/components/Messages/FollowUpQuestions';
import { LikeDislikeButtons } from '@/components/Messages/LikeDislike';
import { ResponseStepsList } from '@/components/Messages/ResponseStepsList';
import { ThoughtStreamComponent } from '@/components/Messages/ThoughtStream';
import { usePreviewStoreState, useConfig } from '@/provider/ChatProvider';

interface AssistantMessageProps {
  message: IAssistantMessage;
  isStreaming?: boolean;
  progressMessage?: string;
  showActions?: boolean;
}

export const AssistantMessage: React.FC<AssistantMessageProps> = memo(
  ({ message, isStreaming = false, progressMessage, showActions = true }) => {
    const setAgentPreviewObject = usePreviewStoreState((s) => s.setAgentPreviewObject);
    const config = useConfig();
    const components = useMarkdownRenderers();
    const chunks = message.assistantChunks;
    const [isFinalStreaming, setIsFinalStreaming] = useState(false);
    const [showThoughts, setShowThoughts] = useState(true);

    const answerCompletedText = config.textConfig?.answerCompletedText || 'Answer completed';
    const helpfulText = config.textConfig?.helpfulText || 'Is this answer helpful?';

    const { streamingThoughts, responseStepsList, nextQuestions } = useMemo(() => {
      const thoughts: string[] = [];
      const steps: any[] = [];
      const nq: string[] = [];

      chunks.forEach((chunk, index) => {
        if (chunk.type === 'streaming' && chunk.content) {
          thoughts.push(chunk.content);
        } else if (chunk.type === 'toolUsed') {
          const action = (chunk as any).message?.action || chunk.action;
          const param = (chunk as any).message?.param || chunk.param;
          steps.push({
            stepIndex: index,
            stepContent: `${action} ${param || chunk.tip || ''}`,
            stepProgress: 'completed',
            stepCallId: chunk.run_id || '',
            chunkObject: chunk,
          });
        } else if (chunk.type === 'next_questions') {
          try {
            const parsed = JSON.parse(chunk.content as string);
            nq.push(...parsed);
          } catch {
            nq.push(chunk.content as string);
          }
        }
      });

      return {
        streamingThoughts: thoughts,
        responseStepsList: steps,
        nextQuestions: nq.slice(0, 5),
      };
    }, [chunks]);

    const getChunkContent = (chunk: IAssistantChunks) => {
      switch (chunk.type) {
        case 'supervisor_streaming':
        case 'security_block': {
          if (!isFinalStreaming) setIsFinalStreaming(true);
          return (
            <ReactMarkdown remarkPlugins={[remarkGfm]} components={components as any}>
              {chunk.content}
            </ReactMarkdown>
          );
        }
        case 'error':
        case 'network_error':
          return <span className="fc-text-sm" style={{ color: 'var(--fc-error)' }}>{chunk.content}</span>;
        default:
          return null;
      }
    };

    const handleThoughtsClick = () => {
      setAgentPreviewObject({
        title: 'Thoughts',
        description: 'Streaming Thoughts',
        image: '',
        type: 'streaming_toughts_preview',
        streamingThoughts,
      });
    };

    return (
      <div className="fc-mt-2 fc-py-3 fc-flex fc-flex-col fc-gap-3 fc-text-sm fc-w-full fc-mr-auto" style={{ color: 'var(--fc-text)' }}>
        {(responseStepsList.length > 0 || isStreaming || streamingThoughts.length > 0) && (
          <div className="fc-flex fc-flex-col fc-gap-4">
            <ResponseStepsList responseStepsList={responseStepsList} />
            {isStreaming ? (
              <ProgressMessageWithAnimation
                progressMessage={progressMessage || CHAT_INPUT_THINKING_MESSAGE}
                isStreaming={isStreaming}
                progressMessageClickHandler={() => setShowThoughts((p) => !p)}
              />
            ) : (
              showActions &&
              streamingThoughts.length > 0 && (
                <button
                  type="button"
                  onClick={handleThoughtsClick}
                  className="fc-group fc-text-sm fc-transition-colors fc-flex fc-items-center fc-gap-2 fc-bg-transparent fc-border-0 fc-cursor-pointer fc-p-0"
                  style={{ color: 'var(--fc-text-secondary)' }}
                >
                  <Brain size={16} className="fc-shrink-0" />
                  <span className="fc-text-sm">Thoughts</span>
                  <ChevronRight size={12} className="fc-hidden group-hover:fc-inline-block" />
                </button>
              )
            )}
          </div>
        )}

        {(!isStreaming || (isStreaming && chunks.length > 1)) && (
          <div
            className={clsx(
              'fc-flex fc-flex-col',
              (isFinalStreaming || !isStreaming) && 'fc-rounded-xl fc-border'
            )}
            style={(isFinalStreaming || !isStreaming) ? { borderColor: 'var(--fc-assistant-border)' } : undefined}
          >
            {!isFinalStreaming && streamingThoughts.length > 0 && (
              <div
                className={clsx(
                  'fc-flex fc-flex-col fc-gap-2 fc-p-4',
                  isStreaming ? 'fc-rounded-3xl fc-border' : '',
                  showThoughts ? 'fc-block' : 'fc-hidden'
                )}
                style={isStreaming ? { borderColor: 'var(--fc-assistant-border)' } : undefined}
              >
                <ThoughtStreamComponent streamingThoughts={streamingThoughts} />
              </div>
            )}
            <div className={`fc-flex fc-flex-col ${isFinalStreaming || !isStreaming ? 'fc-p-4' : ''}`}>
              {chunks.map((chunk, i) => (
                <React.Fragment key={i}>{getChunkContent(chunk)}</React.Fragment>
              ))}
              {!isStreaming && showActions && (
                <div className="fc-flex fc-justify-between fc-items-center fc-border-t fc-pt-4 fc-relative" style={{ borderColor: 'var(--fc-assistant-border)' }}>
                  <div className="fc-flex fc-items-center fc-gap-2">
                    <Sparkles size={16} className="fc-shrink-0" style={{ color: 'var(--fc-success)' }} />
                    <span className="fc-text-xs md:fc-text-sm" style={{ color: 'var(--fc-success)' }}>
                      {answerCompletedText}
                    </span>
                  </div>
                  <div className="fc-flex fc-items-center fc-gap-1 md:fc-gap-2">
                    <span className="fc-hidden md:fc-block md:fc-text-sm" style={{ color: 'var(--fc-text-secondary)' }}>
                      {helpfulText}
                    </span>
                    <LikeDislikeButtons initialReaction={message?.reaction} />
                  </div>
                </div>
              )}
            </div>
            {nextQuestions.length > 0 && !isStreaming && showActions && (
              <FollowUpQuestions nextQuestions={nextQuestions} />
            )}
          </div>
        )}
      </div>
    );
  }
);

AssistantMessage.displayName = 'AssistantMessage';
