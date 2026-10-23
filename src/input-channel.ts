import type { RoutingStatus } from './receiver-routing';
import { routeReady } from './receiver-routing';

export interface ChannelStatus { selected: string; ipcAvailable: boolean; reason: string; epoch: number; }
export function executionReady(channel: ChannelStatus, bindingEpoch: number, routing: RoutingStatus): boolean {
  if (channel.selected === 'taptaploot-ipc') return channel.ipcAvailable && channel.epoch === bindingEpoch;
  return channel.selected === 'message' && routeReady(routing);
}
export function intervalError(minimum: number | string, maximum: number | string, _ipc: boolean): string {
  const low=1;
  const value=(input:number|string) => typeof input==='number' ? input : input.trim()==='' ? NaN : Number(input);
  const min=value(minimum), max=value(maximum);
  if (![min,max].every(value=>Number.isFinite(value)&&Number.isInteger(value)&&value>=low&&value<=60000)||min>max) return `请输入 ${low}～60000 毫秒整数，最小值不能大于最大值。`;
  return '';
}
