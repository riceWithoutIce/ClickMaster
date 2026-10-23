export const keys = [
  { value: 'Space', label: '空格' }, { value: 'Enter', label: '回车' },
  { value: 'ArrowLeft', label: '← 左方向' }, { value: 'ArrowUp', label: '↑ 上方向' },
  { value: 'ArrowRight', label: '→ 右方向' }, { value: 'ArrowDown', label: '↓ 下方向' },
  ...'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789'.split('').map(value => ({ value, label: value })),
];
import type { ReceiverView } from './receiver-routing';
import type { Timing } from './execution-time';
export interface InputStatus {
  timing: Timing;
  repeating: boolean; completed: number;
  channel: string;
  ipc: IpcReceipt | null;
  revision: number; taskId: number; phase: string; busy: boolean; remainingMs: number; message: string;
  summary: string;
  receiver: ReceiverView | null;
  down: Receipt | null; up: Receipt | null;
}
interface Receipt { posted: boolean; error: { reason: string; errorCode: number | null } | null; }
interface IpcReceipt {
  connected: boolean; written: boolean;
  readStatus: 'pending' | 'confirmed' | 'unconfirmed' | 'failed';
  readErrorCode: number | null;
  failure?: { stage: string; reason: string; errorCode: number | null } | null;
}
export function ipcFailureText(receipt: IpcReceipt): string {
  const failure = receipt.failure;
  if (!failure) return '';
  const stages: Record<string, string> = { target: '目标核验', compatibility: '版本核验', 'pipe-create': '创建管道', connect: '连接', client: '客户端核验', write: '单帧写入', authorization: '授权', worker: '执行进程', status: '状态记录', supervision: '执行监督', cleanup: '清理' };
  const reasons: Record<string, string> = {
    'process-unavailable': '无法核验原进程', 'process-exited': '原进程已退出', 'instance-changed': '原实例已变化', 'version-unreadable': '无法读取版本文件', 'unsupported-version': '版本不支持', 'process-enumeration': '进程枚举失败', 'multiple-instances': '存在多个目标实例', 'window-enumeration': '窗口枚举失败', 'window-ambiguous': '目标窗口不唯一', 'window-invalid': '原窗口失效', 'window-identity': '窗口所属实例变化', 'window-hidden': '窗口不可见', 'window-minimized': '窗口已最小化', 'empty-client': '客户区不可用', 'system-call': '系统操作失败', timeout: '超时，未重试', 'client-mismatch': '管道客户端不匹配', 'write-incomplete': '单帧写入不完整', 'setup-failed': '准备失败', 'authorization-failed': '授权未完成', 'worker-exited': '完整写入前退出（具体原因未确认）', 'report-invalid': '状态记录无效', 'cleanup-unconfirmed': '清理未确认', 'random-unavailable': '随机源不可用', 'counter-limit': '计数达到上限',
  };
  return `${stages[failure.stage] ?? '执行'}：${reasons[failure.reason] ?? '失败原因未确认'}${failure.errorCode == null ? '' : `（系统码 ${failure.errorCode}）`}`;
}
export function ipcReadText(receipt: IpcReceipt): string {
  if (receipt.readStatus === 'confirmed') return '已确认（仅传输层）';
  if (receipt.readStatus === 'failed') return `确认失败${receipt.readErrorCode == null ? '' : `（错误码 ${receipt.readErrorCode}）`}，不阻断发送`;
  if (receipt.readStatus === 'unconfirmed') return '未确认，不阻断发送';
  return receipt.written ? '观察中，不影响发送结果' : '未观察';
}
export function inputTitle(status: InputStatus): string {
  if (status.phase === 'countdown') return `倒计时 ${Math.ceil(status.remainingMs / 1000)} 秒`;
  if (status.phase === 'waiting') return `下次等待 ${Math.ceil(status.remainingMs)} 毫秒`;
  return ({ 'target-invalid': '目标失效', expired: '时长已到', idle: '待命', sending: '正在投递', delivered: '已投递', cancelled: '已停止', stopping: '停止中', rejected: '未发送', failed: '执行失败', partial: '部分投递', checking: '正在核验', connecting: '等待 IPC 连接', writing: '正在写入单帧', draining: '已发送 · 收尾中', completed: '已发送', timeout: '已超时', 'cleanup-failed': '清理待确认' } as Record<string, string>)[status.phase] ?? '状态未知';
}
export function receiptText(name: string, receipt: Receipt | null): string {
  if (!receipt) return '';
  if (receipt.posted) return `${name}：已投递`;
  return `${name}：${receipt.error?.reason ?? '未投递'}${receipt.error?.errorCode == null ? '' : `（错误码 ${receipt.error.errorCode}）`}`;
}
