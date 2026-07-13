import React, { useState } from 'react';
import clsx from 'clsx';
import { ChevronDown, ChevronUp, CircleCheck, CircleX, LoaderCircle } from 'lucide-react';
import { useChatStore, usePreviewStoreState, useConfig } from '@/provider/ChatProvider';

interface ResponseStep {
  stepIndex: number;
  stepContent: string;
  stepProgress: 'started' | 'completed' | 'error';
  chunkObject?: {
    name?: string;
    call_id?: string;
  };
}

export const ResponseStepsList: React.FC<{ responseStepsList: ResponseStep[] }> = ({
  responseStepsList,
}) => {
  const [isExpanded, setIsExpanded] = useState(false);
  const setAgentPreviewObject = usePreviewStoreState((s) => s.setAgentPreviewObject);
  const chatStore = useChatStore();
  const config = useConfig();

  const stepsCompletedText = config.textConfig?.stepsCompletedText || 'All Steps Completed';
  const stepsErrorText = config.textConfig?.stepsErrorText || 'Steps Error';
  const processingStepsText = config.textConfig?.processingStepsText || 'Processing Steps...';
  const showDetailsText = config.textConfig?.showDetailsText || 'Show details';
  const hideDetailsText = config.textConfig?.hideDetailsText || 'Hide details';

  if (responseStepsList.length === 0) return null;

  const completedCount = responseStepsList.filter((s) => s.stepProgress === 'completed').length;
  const errorCount = responseStepsList.filter((s) => s.stepProgress === 'error').length;
  const totalCount = responseStepsList.length;
  const allCompleted = completedCount === totalCount;
  const stepsErrorExists = errorCount > 0 && completedCount + errorCount === totalCount;
  const allError = errorCount > 0 && errorCount === totalCount;

  const handleStepClick = (step: ResponseStep) => {
    if (!step.chunkObject) return;
    setAgentPreviewObject({
      title: step.chunkObject.name || 'Tool',
      description: `${step.chunkObject.name} tool execution`,
      image: '',
      type: 'computer',
    });
    const activeThreadId = chatStore.getState().getActiveThreadId();
    if (activeThreadId && step.chunkObject.call_id) {
      chatStore.getState().jumpToCallId(activeThreadId, step.chunkObject.call_id);
    }
  };

  const getStepIcon = () => {
    if (allError) return <CircleX size={18} className="fc-shrink-0" style={{ color: 'var(--fc-error)' }} />;
    if (allCompleted || stepsErrorExists) return <CircleCheck size={18} className="fc-shrink-0" style={{ color: 'var(--fc-success)' }} />;
    return <LoaderCircle size={18} className="fc-animate-spin fc-shrink-0" style={{ color: 'var(--fc-success)' }} />;
  };

  const getStepText = () => {
    if (allError) return stepsErrorText;
    if (allCompleted || stepsErrorExists) return stepsCompletedText;
    return processingStepsText;
  };

  return (
    <div className="fc-rounded-xl fc-border fc-px-2 fc-py-1 fc-shadow-sm fc-flex fc-flex-col fc-gap-2" style={{ backgroundColor: 'var(--fc-bg)', borderColor: 'var(--fc-border)' }}>
      <div className="fc-w-full fc-flex fc-items-center fc-justify-between fc-cursor-default">
        <div className="fc-flex fc-items-center fc-gap-2">
          {getStepIcon()}
          <span className="fc-font-medium fc-text-xs" style={{ color: allError ? 'var(--fc-error)' : 'var(--fc-success)' }}>
            {getStepText()}
          </span>
        </div>
        <div className="fc-flex fc-items-center fc-gap-3">
          <span className="fc-text-xs" style={{ color: 'var(--fc-text-secondary)' }}>{completedCount}/{totalCount}</span>
          <button
            type="button"
            onClick={() => setIsExpanded(!isExpanded)}
            className="fc-flex fc-items-center fc-gap-1 fc-transition-colors fc-cursor-pointer fc-p-1.5 fc-rounded-lg fc-bg-transparent fc-border-0"
            style={{ color: 'var(--fc-text-secondary)' }}
          >
            {isExpanded ? <ChevronUp size={18} /> : <ChevronDown size={18} />}
            <span className="fc-text-sm">{isExpanded ? hideDetailsText : showDetailsText}</span>
          </button>
        </div>
      </div>

      {isExpanded && (
        <div className="fc-flex fc-flex-col fc-gap-3 fc-px-2 md:fc-px-4">
          {responseStepsList.map((step) => (
            <button
              type="button"
              onClick={() => handleStepClick(step)}
              key={step.stepIndex}
              className={clsx(
                step.stepProgress === 'error' ? 'fc-line-through' : 'fc-cursor-pointer',
                'fc-flex fc-items-center fc-gap-1 fc-p-1.5 fc-rounded fc-transition-colors fc-text-left fc-w-full fc-bg-transparent fc-border-0'
              )}
              style={{ color: step.stepProgress === 'error' ? 'var(--fc-text-secondary)' : 'var(--fc-text)' }}
              disabled={!step.chunkObject}
            >
              {step.stepProgress === 'started' ? (
                <LoaderCircle size={18} className="fc-animate-spin fc-shrink-0" style={{ color: 'var(--fc-success)' }} />
              ) : step.stepProgress === 'completed' ? (
                <CircleCheck size={18} className="fc-shrink-0" style={{ color: 'var(--fc-success)' }} />
              ) : (
                <CircleX size={18} className="fc-shrink-0" style={{ color: 'var(--fc-text-secondary)' }} />
              )}
              <span className="fc-text-xs fc-leading-relaxed fc-line-clamp-1">
                {step.stepContent}
              </span>
            </button>
          ))}
        </div>
      )}
    </div>
  );
};
