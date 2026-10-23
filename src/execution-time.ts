export interface Timing {
  limitSeconds: number | null; elapsedSeconds: number; remainingSeconds: number | null;
  started: boolean; endReason: string | null; triggerReason: string | null;
}
export function durationError(mode: string, hours: number|string, minutes: number|string, seconds: number|string): string {
  if (mode === 'permanent') return '';
  const values=[hours, minutes, seconds].map(v => typeof v === 'string' && v.trim() === '' ? NaN : Number(v));
  if (mode !== 'timed' || !values.every((v,i)=>Number.isFinite(v) && Number.isInteger(v) && v>=0 && v<=(i===0?99:59)) || values[0]*3600+values[1]*60+values[2]===0) return '限时请输入 0～99 小时、0～59 分和秒，总时长至少 1 秒。';
  return '';
}
export function clockText(seconds:number):string {
  return [Math.floor(seconds/3600),Math.floor(seconds/60)%60,seconds%60].map(v=>String(v).padStart(2,'0')).join(':');
}
export function timingText(timing:Timing, repeating:boolean, taskId:number):string {
  if (!taskId) return '开始后准备 3 秒 · F8 停止';
  if (!timing.started && !timing.endReason) return '准备阶段尚未计时 · F8 停止';
  const elapsed=`已运行 ${clockText(timing.elapsedSeconds)}`;
  if (timing.endReason) return `${timing.triggerReason && timing.triggerReason !== timing.endReason ? timing.triggerReason+' · ' : ''}${timing.endReason} · 耗时 ${clockText(timing.elapsedSeconds)}`;
  return repeating ? timing.limitSeconds===null ? `永久 · ${elapsed}` : `${elapsed} · 剩余 ${clockText(timing.remainingSeconds??0)}` : elapsed;
}
