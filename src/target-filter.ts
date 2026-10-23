export interface TargetView {
  token: string;
  title: string;
  exe: string;
  pid: number;
  status: 'visible' | 'minimized' | 'hidden';
}

export function filterTargets(windows: TargetView[], text: string): TargetView[] {
  const term = text.trim().toLowerCase();
  return windows.filter(window => !term || [window.title, window.exe, String(window.pid)]
    .some(value => value.toLowerCase().includes(term)));
}

export function selectedTarget(windows: TargetView[], token: string): TargetView | undefined {
  return windows.find(window => window.token === token);
}

export function displayTitle(window: TargetView): string { return window.title || '无标题窗口'; }
export function displayStatus(window: TargetView): string {
  return { visible: '可见', minimized: '最小化', hidden: '隐藏' }[window.status];
}
