export interface ReceiverView {
  token: string; exe: string; pid: number; class: string; width: number; height: number; isSelf: boolean;
}
export interface RoutingStatus {
  epoch: number; phase: string; message: string; selected: ReceiverView | null; candidates: ReceiverView[];
}
export function receiverLabel(receiver: ReceiverView): string {
  return `${receiver.isSelf ? '绑定窗口自身' : '内容窗口'} · ${receiver.exe} · PID ${receiver.pid} · ${receiver.class} · ${receiver.width}×${receiver.height}`;
}
export function routeReady(route: RoutingStatus): boolean {
  return !!route.selected && ['self', 'automatic', 'manual'].includes(route.phase);
}
export function routeMode(phase: string): string {
  return ({ self: '自身', automatic: '自动', manual: '手动', choose: '待选择', invalid: '已失效', unbound: '未绑定' } as Record<string, string>)[phase] ?? '不可用';
}
