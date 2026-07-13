import { useEffect, useState } from 'react';
import { useConfig } from '@/provider/ChatProvider';

function getSystemScheme(): 'light' | 'dark' {
  if (typeof window === 'undefined') return 'light';
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

export function useTheme() {
  const config = useConfig();
  const configTheme = config.theme;

  const [systemScheme, setSystemScheme] = useState<'light' | 'dark'>(getSystemScheme);

  useEffect(() => {
    if (configTheme && configTheme !== 'auto') return;
    const mq = window.matchMedia('(prefers-color-scheme: dark)');
    const handler = (e: MediaQueryListEvent) => {
      setSystemScheme(e.matches ? 'dark' : 'light');
    };
    mq.addEventListener('change', handler);
    return () => mq.removeEventListener('change', handler);
  }, [configTheme]);

  const colorScheme: 'light' | 'dark' =
    configTheme === 'light' || configTheme === 'dark' ? configTheme : systemScheme;

  return { colorScheme };
}
