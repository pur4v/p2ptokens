import React from 'react';
import clsx from 'clsx';
import { Play, SkipBack, SkipForward } from 'lucide-react';
import { IToolType } from '@/types';
import { useChatStoreState, useChatStore, useConfig } from '@/provider/ChatProvider';
import { ToolContentRenderer } from '@/components/SidePanel/ToolContentRenderer';

const EMPTY_TRACKER: any[] = [];

export const ProgressPreview: React.FC = () => {
  const chatStore = useChatStore();
  const config = useConfig();
  const activeThreadId = useChatStoreState((s) => s.activeThreadId);

  const liveText = config.textConfig?.liveText || 'live';

  const progressTracker = useChatStoreState((s) => {
    if (!activeThreadId || !s.threadsById[activeThreadId]) return EMPTY_TRACKER;
    return s.threadsById[activeThreadId].computerState.progressTracker;
  });

  const progress = useChatStoreState((s) => {
    if (!activeThreadId || !s.threadsById[activeThreadId]) return 0;
    return s.threadsById[activeThreadId].computerState.activeIndex;
  });

  if (!activeThreadId || progressTracker.length === 0) return null;

  const currentItem = progressTracker[progress];
  if (!currentItem) return null;

  const headTitle: Record<string, string> = {
    search_web_tool:
      (progressTracker[progress]?.detail?.content as any)?.query ?? '',
  };

  const titleText =
    headTitle[currentItem.name] ?? currentItem.message?.action ?? 'Agent Tool';

  return (
    <div className="fc-flex fc-px-4 fc-pt-4 fc-flex-1 fc-min-h-0 fc-w-full fc-relative">
      <div className="fc-flex fc-flex-col fc-w-full">
        {/* Tool content area */}
        <div className="fc-border fc-flex fc-flex-col fc-min-h-0 fc-flex-1 fc-rounded-t-lg fc-overflow-hidden" style={{ borderColor: 'var(--fc-border)', backgroundColor: 'var(--fc-bg)' }}>
          {/* Title bar */}
          <div className="fc-w-full fc-h-9 fc-border-b fc-rounded-t-lg fc-flex fc-items-center fc-justify-center" style={{ borderColor: 'var(--fc-border)' }}>
            <p className="fc-text-sm fc-line-clamp-1 fc-px-2" style={{ color: 'var(--fc-text-secondary)' }}>{titleText}</p>
          </div>

          {/* Content */}
          <div className="fc-flex-1 fc-min-h-0 fc-w-full fc-flex fc-flex-col fc-overflow-y-auto">
            <div className="fc-flex-1 fc-min-h-0 fc-w-full fc-relative">
              <ToolContentRenderer progressItem={currentItem} />
              {progress !== progressTracker.length - 1 && (
                <div
                  className={clsx(
                    'fc-bottom-4 fc-w-full fc-flex fc-justify-center',
                    currentItem.name === 'search_web_tool' ? 'fc-sticky' : 'fc-absolute'
                  )}
                >
                  <button
                    type="button"
                    className="fc-py-2 fc-px-3 fc-rounded-3xl fc-cursor-pointer fc-text-xs fc-flex fc-items-center fc-gap-1 fc-border fc-shadow-sm"
                    style={{ backgroundColor: 'var(--fc-bg)', color: 'var(--fc-text)', borderColor: 'var(--fc-border)' }}
                    onClick={() => chatStore.getState().jumpToLive(activeThreadId)}
                  >
                    <Play size={14} />
                    <span>Jump to live</span>
                  </button>
                </div>
              )}
            </div>
          </div>
        </div>

        {/* Navigation bar */}
        <div className="fc-mt-auto fc-w-full fc-h-11 fc-border fc-rounded-b-lg" style={{ backgroundColor: 'var(--fc-bg)', borderColor: 'var(--fc-border)' }}>
          <div className="fc-flex fc-gap-2 fc-items-center fc-h-full fc-px-4">
            <div className="fc-flex fc-gap-2">
              <button
                type="button"
                className={clsx(
                  'fc-bg-transparent fc-border-0 fc-p-0',
                  progress > 0 ? 'fc-cursor-pointer' : 'fc-cursor-not-allowed fc-opacity-40'
                )}
                style={{ color: 'var(--fc-text-secondary)' }}
                onClick={() => chatStore.getState().decrementActiveIndex(activeThreadId)}
                disabled={progress <= 0}
              >
                <SkipBack size={16} />
              </button>
              <button
                type="button"
                className={clsx(
                  'fc-bg-transparent fc-border-0 fc-p-0',
                  progress < progressTracker.length - 1
                    ? 'fc-cursor-pointer'
                    : 'fc-cursor-not-allowed fc-opacity-40'
                )}
                style={{ color: 'var(--fc-text-secondary)' }}
                onClick={() => chatStore.getState().incrementActiveIndex(activeThreadId)}
                disabled={progress >= progressTracker.length - 1}
              >
                <SkipForward size={16} />
              </button>
            </div>
            <input
              type="range"
              min={0}
              max={progressTracker.length - 1}
              value={progress}
              onChange={(e) =>
                chatStore.getState().setActiveIndex(activeThreadId, Number(e.target.value))
              }
              className="fc-w-full fc-slider"
            />
            <div className="fc-flex fc-items-center fc-gap-1.5 fc-shrink-0">
              <div className="fc-w-2 fc-h-2 fc-rounded-full" style={{ backgroundColor: 'var(--fc-success)' }} />
              <span className="fc-text-sm" style={{ color: 'var(--fc-text)' }}>{liveText}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
