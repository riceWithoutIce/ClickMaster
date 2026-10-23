<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { durationError, timingText } from './execution-time';
import NavIcon from './NavIcon.vue';
import appIcon from './assets/app-icon.png';
import { displayStatus, displayTitle, filterTargets, selectedTarget, type TargetView } from './target-filter';
import { overlayPosition } from './overlay-position';
import { keys, inputTitle, receiptText, ipcReadText, ipcFailureText, type InputStatus } from './single-input';
import { receiverLabel, routeMode, type RoutingStatus } from './receiver-routing';
import { executionReady as channelReady, intervalError, type ChannelStatus } from './input-channel';

interface AppInfo { version: string; minimizeKeepTaskbar: boolean; trayAvailable: boolean; warning: string | null; closePending: boolean; }
interface BindingStatus { revision: number; epoch: number; bound: TargetView | null; reason: string | null; routing: RoutingStatus; }
interface OverlayStatus { revision: number; enabled: boolean; visible: boolean; message: string; }
interface TargetList { windows: TargetView[]; inaccessible: number; binding: BindingStatus; }
type Page = 'target' | 'input' | 'execution' | 'settings' | 'about';
const controlPages = [
  { id: 'target', title: '目标窗口', icon: 'window' },
  { id: 'input', title: '输入方式', icon: 'mouse' },
  { id: 'execution', title: '执行参数', icon: 'timer' },
] as const;
const appPages = [{ id: 'settings', title: '设置', icon: 'settings' }, { id: 'about', title: '关于', icon: 'info' }] as const;
const page = ref<Page>('target');
const pageTitle = computed(() => [...controlPages, ...appPages].find(item => item.id === page.value)!.title);
type InfoKind = 'help' | 'interval' | 'details' | 'diagnostics';
const infoKind = ref<InfoKind>('help');
const infoDialog = ref<HTMLDialogElement | null>(null);
const infoClose = ref<HTMLButtonElement | null>(null);
const moreButton = ref<HTMLButtonElement | null>(null);
const moreOpen = ref(false);
let infoPreviousFocus: HTMLElement | null = null;
const infoTitle = computed(() => ({ help: `${pageTitle.value}帮助`, interval: '间隔说明', details: '窗口详情', diagnostics: '诊断信息' })[infoKind.value]);
async function openInfo(kind: InfoKind) {
  infoPreviousFocus = kind === 'diagnostics' ? moreButton.value : document.activeElement instanceof HTMLElement ? document.activeElement : null;
  moreOpen.value = false;
  infoKind.value = kind;
  await nextTick();
  if (infoDialog.value && !infoDialog.value.open) infoDialog.value.showModal();
  infoClose.value?.focus();
}
function closeInfo() { infoDialog.value?.close(); }
function restoreInfoFocus() { infoPreviousFocus?.focus(); }
function dismissMore(event: MouseEvent) { if (!(event.target instanceof Element) || !event.target.closest('.more-wrap')) moreOpen.value = false; }
function leaveMore(event: FocusEvent) { if (!(event.currentTarget instanceof Element) || !(event.relatedTarget instanceof Node) || !event.currentTarget.contains(event.relatedTarget)) moreOpen.value = false; }
const mode = ref('left');
const center = ref(true);
const coordinateX = ref<number | string>('');
const coordinateY = ref<number | string>('');
const keyboardKey = ref('Space');
const inputStatus = ref<InputStatus>({ timing: { limitSeconds:null,elapsedSeconds:0,remainingSeconds:null,started:false,endReason:null,triggerReason:null }, repeating: false, completed: 0, channel: 'message', ipc: null, revision: 0, taskId: 0, phase: 'idle', busy: false, remainingMs: 0, message: '待命；开始前注册全局停止 F8。', summary: '', receiver: null, down: null, up: null });
const durationMode=ref('permanent');
const durationHours=ref<number|string>(0), durationMinutes=ref<number|string>(0), durationSeconds=ref<number|string>(30);
const durationProblem=computed(()=>durationError(durationMode.value,durationHours.value,durationMinutes.value,durationSeconds.value));
const progressText=computed(()=>timingText(inputStatus.value.timing,inputStatus.value.repeating,inputStatus.value.taskId));
const criticalPhase=computed(()=>['target-invalid','failed','partial','rejected','timeout','cleanup-failed'].includes(inputStatus.value.phase));
const statusTone = computed(() => {
  const phase = inputStatus.value.phase;
  if (['target-invalid', 'failed', 'partial', 'rejected', 'timeout', 'cleanup-failed'].includes(phase)) return 'error';
  if (phase === 'stopping') return 'stopping';
  if (['countdown', 'checking', 'connecting', 'writing'].includes(phase)) return 'preparing';
  if (inputStatus.value.busy || ['sending', 'waiting', 'draining'].includes(phase)) return 'running';
  return 'idle';
});
const statusLabel = computed(() => {
  if (statusTone.value === 'running') return '运行中';
  if (statusTone.value === 'preparing') return '准备中';
  if (statusTone.value === 'stopping') return '停止中';
  if (statusTone.value === 'error') return inputTitle(inputStatus.value);
  if (inputStatus.value.phase === 'completed') return '已完成';
  if (inputStatus.value.phase === 'cancelled') return '已停止';
  return '待命';
});
const intervalMin=ref<number|string>(2000);
const intervalMax=ref<number|string>(2000);
const intervalProblem=computed(()=>intervalError(intervalMin.value,intervalMax.value,isIpc.value));
const intervalHint = computed(() => intervalProblem.value ? '' : Number(intervalMin.value) === Number(intervalMax.value) ? `每次操作后等待 ${Number(intervalMin.value)} 毫秒。` : `在 ${Number(intervalMin.value)}～${Number(intervalMax.value)} 毫秒之间随机等待。`);
const inputBusy = ref(false);
const channelBusy = ref(false);
const channelStatus = ref<ChannelStatus>({ selected: 'message', ipcAvailable: false, reason: '请绑定支持版本的 TapTapLoot 窗口。', epoch: 0 });
const isIpc = computed(() => channelStatus.value.selected === 'taptaploot-ipc');
const channelLabel = computed(() => isIpc.value ? 'TapTapLoot 专用 IPC' : '窗口消息 PostMessage');
const executionReady = computed(() => channelReady(channelStatus.value, bindingStatus.value.epoch, bindingStatus.value.routing));
const inputError = ref('');
interface PerformanceView { active: boolean; elapsedSeconds: number; message: string; fileStem: string | null; complete: boolean; }
const performanceView=ref<PerformanceView>({active:false,elapsedSeconds:0,message:'性能采集默认关闭。',fileStem:null,complete:false});
const performanceBusy=ref(false);
async function capturePerformance(start:boolean) {
  if(performanceBusy.value)return;
  performanceBusy.value=true;
  try { performanceView.value=await invoke<PerformanceView>(start?'start_performance_capture':'stop_performance_capture'); }
  catch { performanceView.value.message='无法开始或结束性能采集，请稍后重试；连点状态不受影响。'; }
  finally { performanceBusy.value=false; }
}
const inputLocked = computed(() => inputBusy.value || channelBusy.value || inputStatus.value.busy);
async function changeChannel(event: Event) {
  const element = event.target as HTMLSelectElement;
  if (inputLocked.value || targetBusy.value || overlayBusy.value) { element.value = channelStatus.value.selected; return; }
  channelBusy.value = true; inputError.value = '';
  if (overlayUpdate) clearTimeout(overlayUpdate);
  try { channelStatus.value = await invoke<ChannelStatus>('select_channel', { channel: element.value }); await pollBinding(); }
  catch (error) { inputError.value = message(error, '切换通道失败，原通道已保留。'); }
  finally { element.value = channelStatus.value.selected; channelBusy.value = false; }
}
function applyInput(status: InputStatus) { if (status.revision >= inputStatus.value.revision) inputStatus.value = status; }
async function startSingle() {
  await startTask(false);
}
async function startRepeat() { await startTask(true); }
async function startTask(repeating:boolean) {
  if (inputLocked.value || targetBusy.value || overlayBusy.value || !bindingStatus.value.bound || !executionReady.value) return;
  if (repeating && (intervalProblem.value || durationProblem.value)) {inputError.value=intervalProblem.value || durationProblem.value;return;}
  inputBusy.value = true; inputError.value = '';
  const request = { mode: mode.value, key: keyboardKey.value, ...overlayPosition(center.value, coordinateX.value, coordinateY.value) };
  try { applyInput(await invoke<InputStatus>(repeating?'start_repeat_input':'start_single_input', { channel: channelStatus.value.selected, request, expectedEpoch: bindingStatus.value.epoch, expectedRouteEpoch: bindingStatus.value.routing.epoch, ...(repeating?{interval:{min:Number(intervalMin.value),max:Number(intervalMax.value)},duration:{mode:durationMode.value,hours:Number(durationHours.value),minutes:Number(durationMinutes.value),seconds:Number(durationSeconds.value)}}:{}) })); }
  catch (error) { inputError.value = message(error, '单次测试请求失败；不会自动重试。'); }
  finally { inputBusy.value = false; }
}
async function cancelSingle() {
  if (inputBusy.value) return;
  inputBusy.value = true; inputError.value = '';
  try { applyInput(await invoke<InputStatus>('cancel_single_input')); }
  catch (error) { inputError.value = message(error, '无法确认取消状态，请重试或退出程序。'); }
  finally { inputBusy.value = false; }
}
const keepTaskbar = ref(false);
const saving = ref(false);
const initialized = ref(false);
const version = ref('正在连接…');
const initializationWarning = ref('');
const notice = ref('');
const noticeError = ref(false);
const isMouse = computed(() => mode.value !== 'keyboard');
const windows = ref<TargetView[]>([]);
const filterText = ref('');
const includeHidden = ref(false);
const filteredWindows = computed(() => filterTargets(windows.value, filterText.value));
const selectedToken = ref('');
const selected = computed(() => selectedTarget(filteredWindows.value, selectedToken.value));
watch(filteredWindows, () => { if (!selected.value) selectedToken.value = ''; }, { flush: 'sync' });
const bindingStatus = ref<BindingStatus>({ revision: 0, epoch: 0, bound: null, reason: null, routing: { epoch: 0, phase: 'unbound', message: '', selected: null, candidates: [] } });
const receiverToken = ref('');
const receiverSummary = computed(() => bindingStatus.value.routing.selected ? receiverLabel(bindingStatus.value.routing.selected) : routeMode(bindingStatus.value.routing.phase));
async function selectReceiver() {
  if (targetBusy.value || inputLocked.value || !receiverToken.value) return;
  targetBusy.value = true; targetError.value = '';
  try { applyBinding(await invoke<BindingStatus>('choose_receiver', { token: receiverToken.value, expectedEpoch: bindingStatus.value.epoch })); }
  catch (error) { targetError.value = message(error, '接收窗口选择失败，请重新绑定。'); await pollBinding(); }
  finally { targetBusy.value = false; }
}
const overlayStatus = ref<OverlayStatus>({ revision: 0, enabled: false, visible: false, message: '坐标预览已关闭。' });
const overlayBusy = ref(false);
const overlayError = ref('');
let overlayUpdate: ReturnType<typeof setTimeout> | undefined;
function applyOverlay(status: OverlayStatus) {
  if (status.revision >= overlayStatus.value.revision) {
    overlayStatus.value = status;
    if (!status.enabled && overlayUpdate) clearTimeout(overlayUpdate);
  }
}
async function updateOverlay(enabled: boolean) {
  if (overlayBusy.value || inputLocked.value || isIpc.value) return;
  if (overlayUpdate) clearTimeout(overlayUpdate);
  overlayBusy.value = true; overlayError.value = '';
  const epoch = bindingStatus.value.epoch;
  const position = overlayPosition(center.value, coordinateX.value, coordinateY.value);
  try { applyOverlay(await invoke<OverlayStatus>('set_overlay', { enabled, expectedEpoch: epoch, position })); }
  catch (error) { overlayError.value = message(error, '坐标预览更新失败，请关闭后重试。'); }
  finally {
    overlayBusy.value = false;
    if (enabled && overlayStatus.value.enabled && !isMouse.value) { void updateOverlay(false); }
    else if (enabled && overlayStatus.value.enabled && epoch === bindingStatus.value.epoch
      && JSON.stringify(position) !== JSON.stringify(overlayPosition(center.value, coordinateX.value, coordinateY.value))) scheduleOverlay();
  }
}
function scheduleOverlay() {
  if (overlayUpdate) clearTimeout(overlayUpdate);
  const epoch = bindingStatus.value.epoch;
  if (overlayStatus.value.enabled && !overlayBusy.value && !inputLocked.value && !isIpc.value) overlayUpdate = setTimeout(() => {
    if (overlayStatus.value.enabled && bindingStatus.value.epoch === epoch && !targetBusy.value && !inputLocked.value && !isIpc.value) void updateOverlay(isMouse.value);
  }, 120);
}
watch([center, coordinateX, coordinateY, isMouse], scheduleOverlay);
const targetLoading = ref(false);
const targetBusy = ref(false);
const targetLoaded = ref(false);
const targetError = ref('');
const inaccessible = ref(0);
const closeDialog = ref<HTMLDialogElement | null>(null);
const cancelButton = ref<HTMLButtonElement | null>(null);
const closeBusy = ref(false);
const closeError = ref('');
let previousFocus: HTMLElement | null = null;
const unlisteners: UnlistenFn[] = [];
let bindingTimer: ReturnType<typeof setInterval> | undefined;
let disposed = false;
function applyBinding(status: BindingStatus) {
  if (status.revision >= bindingStatus.value.revision) {
    if (status.epoch !== bindingStatus.value.epoch && overlayUpdate) clearTimeout(overlayUpdate);
    bindingStatus.value = status;
    if (!status.routing.candidates.some(item => item.token === receiverToken.value)) receiverToken.value = status.routing.selected?.token ?? '';
  }
}
function message(error: unknown, fallback: string): string { return typeof error === 'string' ? error : fallback; }
async function pollBinding() {
  try { performanceView.value=await invoke<PerformanceView>('get_performance_status'); } catch { /* Telemetry is independent of input controls. */ }
  try { applyBinding(await invoke<BindingStatus>('get_binding_status')); }
  catch { targetError.value = '无法读取绑定状态，请刷新重试。'; }
  try { applyOverlay(await invoke<OverlayStatus>('get_overlay_status')); } catch { /* Preview failure is isolated from binding. */ }
  try { applyInput(await invoke<InputStatus>('get_input_status')); } catch { inputError.value = '无法读取单次任务状态，请重试或退出程序。'; }
  try { channelStatus.value = await invoke<ChannelStatus>('get_channel_status'); } catch { /* Sending always repeats backend compatibility checks. */ }
}
async function refreshTargets() {
  if (targetLoading.value || targetBusy.value || inputLocked.value) return;
  targetLoading.value = true;
  targetError.value = '';
  selectedToken.value = '';
  windows.value = [];
  try {
    const list = await invoke<TargetList>('refresh_targets', { includeHidden: includeHidden.value });
    windows.value = list.windows;
    inaccessible.value = list.inaccessible;
    applyBinding(list.binding);
    targetLoaded.value = true;
  } catch (error) {
    targetError.value = message(error, '刷新失败，请重试。');
    await pollBinding();
  } finally { targetLoading.value = false; }
}
async function changeTargetRange(event: Event) {
  if (inputLocked.value) return;
  includeHidden.value = (event.target as HTMLInputElement).checked;
  await refreshTargets();
}
async function bindSelected() {
  if (!selected.value || targetLoading.value || targetBusy.value || inputLocked.value) return;
  targetBusy.value = true;
  targetError.value = '';
  try { applyBinding(await invoke<BindingStatus>('bind_target', { token: selectedToken.value })); }
  catch (error) { targetError.value = message(error, '绑定失败，请刷新后重新选择。'); await pollBinding(); }
  finally { targetBusy.value = false; }
}
async function unbindTarget() {
  if (inputLocked.value || targetBusy.value) return;
  targetBusy.value = true;
  targetError.value = '';
  try { applyBinding(await invoke<BindingStatus>('unbind_target')); }
  catch (error) { targetError.value = message(error, '解除绑定失败，请重试。'); }
  finally { targetBusy.value = false; }
}
async function showCloseChoice() {
  await nextTick();
  if (!closeDialog.value || closeDialog.value.open) return;
  previousFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
  closeError.value = '';
  closeDialog.value.showModal();
  cancelButton.value?.focus();
}
async function resolveClose(action: 'cancel' | 'tuck' | 'exit') {
  if (closeBusy.value) return;
  closeBusy.value = true;
  closeError.value = '';
  try {
    await invoke('resolve_close', { action });
    closeDialog.value?.close();
    previousFocus?.focus();
  } catch (error) { closeError.value = message(error, '操作失败，窗口保持打开，请重试或取消。'); }
  finally { closeBusy.value = false; }
}
onMounted(async () => {
  try {
    unlisteners.push(await listen<BindingStatus>('binding-changed', event => applyBinding(event.payload)));
    unlisteners.push(await listen<OverlayStatus>('overlay-changed', event => applyOverlay(event.payload)));
    unlisteners.push(await listen<InputStatus>('input-changed', event => applyInput(event.payload)));
    unlisteners.push(await listen('close-choice-requested', () => { void showCloseChoice(); }));
    unlisteners.push(await listen<string>('shell-warning', event => { initializationWarning.value = event.payload; }));
  } catch { initializationWarning.value = '窗口通知初始化失败；请使用托盘退出，或重新启动程序。'; }
  try {
    const info = await invoke<AppInfo>('get_app_info');
    version.value = info.version;
    keepTaskbar.value = info.minimizeKeepTaskbar;
    initialized.value = true;
    if (info.warning) initializationWarning.value = info.warning;
    if (info.closePending) await showCloseChoice();
  } catch {
    version.value = '连接失败';
    initializationWarning.value = '初始化异常：无法读取应用信息，请重新启动程序。';
  }
  if (disposed) { unlisteners.splice(0).forEach(stop => stop()); return; }
  await refreshTargets();
  await pollBinding();
  bindingTimer = setInterval(() => { void pollBinding(); }, 1500);
});
onUnmounted(() => { disposed = true; unlisteners.splice(0).forEach(stop => stop()); if (bindingTimer) clearInterval(bindingTimer); if (overlayUpdate) clearTimeout(overlayUpdate); });
async function savePreference(event: Event) {
  if (inputLocked.value) { (event.target as HTMLInputElement).checked = keepTaskbar.value; return; }
  const desired = (event.target as HTMLInputElement).checked;
  saving.value = true;
  notice.value = '';
  try {
    await invoke('set_minimize_preference', { keepTaskbar: desired });
    keepTaskbar.value = desired;
    notice.value = '偏好已保存';
    noticeError.value = false;
  } catch (error) {
    notice.value = typeof error === 'string' ? error : '偏好保存失败，已保留上次设置。';
    noticeError.value = true;
  } finally {
    (event.target as HTMLInputElement).checked = keepTaskbar.value;
    saving.value = false;
  }
}
</script>

<template>
  <div class="app-shell" @click="dismissMore">
    <aside class="sidebar">
      <div class="brand"><img :src="appIcon" alt="ClickMaster" title="ClickMaster"></div>
      <nav aria-label="连点控制"><div class="subnav"><button v-for="item in controlPages" :key="item.id" :aria-label="item.title" :title="item.title" :aria-current="page === item.id ? 'page' : undefined" @click="page = item.id"><NavIcon :name="item.icon"/></button></div></nav>
      <nav class="bottom-nav" aria-label="应用"><button v-for="item in appPages" :key="item.id" :aria-label="item.title" :title="item.title" :aria-current="page === item.id ? 'page' : undefined" @click="page = item.id"><NavIcon :name="item.icon"/></button></nav>
    </aside>
    <main class="main-area">
      <header class="target-summary"><span class="target-caption"><NavIcon name="window"/><span>{{ bindingStatus.bound?.exe ?? '尚未绑定窗口' }}</span></span><span class="status" :class="`status-${statusTone}`"><i aria-hidden="true"></i>{{ statusLabel }}</span></header>
      <div v-if="initializationWarning || notice || targetError || bindingStatus.reason || inputError" class="notice-area" aria-live="polite">
        <p v-if="initializationWarning" class="notice error">{{ initializationWarning }}</p><p v-if="notice" class="notice" :class="{ error: noticeError }" role="status">{{ notice }}</p><p v-if="targetError" class="notice error">{{ targetError }}</p><p v-if="inputError" class="notice error">{{ inputError }}</p><p v-if="bindingStatus.reason" class="notice error">{{ bindingStatus.reason }}</p>
      </div>
      <div class="page-scroll">
        <header class="page-header"><h1>{{ pageTitle }}</h1><button class="text-button icon-button" aria-label="页面帮助" title="查看帮助" @click="openInfo('help')"><NavIcon name="help"/></button></header>
        <section v-if="page === 'target'" class="panel" aria-label="目标窗口">
          <span class="section-label">查找应用</span>
          <div class="search-row"><label class="search-field"><NavIcon name="search"/><input id="target-filter" v-model="filterText" type="search" placeholder="搜索应用名称或窗口标题" aria-label="搜索应用名称、窗口标题或 PID" autocomplete="off"></label><button :disabled="targetLoading || targetBusy || inputLocked" @click="refreshTargets"><NavIcon name="refresh"/>{{ targetLoading ? '刷新中' : '刷新' }}</button></div>
          <label class="range-option target-range"><input :checked="includeHidden" type="checkbox" :disabled="targetLoading || targetBusy || inputLocked" @change="changeTargetRange">显示后台窗口</label>
          <label class="field target-selection" for="target"><span>选择窗口</span><select id="target" v-model="selectedToken" :disabled="targetLoading || targetBusy || inputLocked || !filteredWindows.length"><option value="">{{ targetLoading ? '正在读取窗口…' : '请选择目标窗口' }}</option><option v-for="window in filteredWindows" :key="window.token" :value="window.token">{{ displayTitle(window) }} · {{ window.exe }}</option></select></label>
          <p v-if="targetLoaded && !targetLoading && !windows.length && !targetError" class="helper">没有可绑定的窗口，请打开目标应用后刷新。</p><p v-else-if="targetLoaded && !targetLoading && !filteredWindows.length && !targetError" class="helper">未找到匹配的窗口，试试其他关键词。</p>
          <div class="bind-row"><button class="text-button" @click="openInfo('details')">窗口详情<NavIcon name="chevron"/></button><div class="binding-actions"><span v-if="bindingStatus.bound" class="binding-label"><NavIcon name="check"/>已绑定</span><button v-if="bindingStatus.bound" class="text-button" :disabled="targetBusy || targetLoading || inputLocked" @click="unbindTarget">解除绑定</button><button v-if="!bindingStatus.bound || (selected && selected.token !== bindingStatus.bound.token)" class="primary" :disabled="!selected || targetBusy || targetLoading || inputLocked" @click="bindSelected">{{ targetBusy ? '处理中…' : '绑定窗口' }}</button></div></div>
        </section>
        <section v-else-if="page === 'input'" class="panel" aria-label="输入方式">
          <label class="field" for="channel"><span>执行方式</span><select id="channel" :value="channelStatus.selected" :disabled="inputLocked || targetBusy || overlayBusy" @change="changeChannel"><option value="message">窗口消息模式</option><option value="taptaploot-ipc" :disabled="!channelStatus.ipcAvailable">TapTapLoot 专用模式</option></select></label>
          <p v-if="isIpc" class="mode-description"><NavIcon name="check"/>使用 TapTapLoot 专用连接执行。</p>
          <template v-else><div class="line"></div><fieldset class="mode-fieldset" :disabled="inputLocked"><legend>输入类型</legend><div class="mode-switch"><label v-for="item in [{ id: 'left', label: '鼠标左键' }, { id: 'right', label: '鼠标右键' }, { id: 'keyboard', label: '键盘按键' }]" :key="item.id" :class="{ active: mode === item.id }"><input v-model="mode" type="radio" name="mode" :value="item.id"><span>{{ item.label }}</span></label></div></fieldset>
            <div v-if="isMouse" class="mode-content"><label class="field" for="position"><span>点击位置</span><select id="position" v-model="center" :disabled="inputLocked"><option :value="true">窗口内容区中心</option><option :value="false">指定坐标</option></select></label><div v-if="!center" class="coordinates"><label class="field">X<input v-model="coordinateX" type="number" :disabled="inputLocked" min="0" placeholder="待设置"></label><label class="field">Y<input v-model="coordinateY" type="number" :disabled="inputLocked" min="0" placeholder="待设置"></label></div></div>
            <label v-else class="field" for="key"><span>选择按键</span><select id="key" v-model="keyboardKey" :disabled="inputLocked"><option v-for="key in keys" :key="key.value" :value="key.value">{{ key.label }}</option></select></label>
          </template>
        </section>
        <section v-else-if="page === 'execution'" class="panel" aria-label="执行参数">
          <div class="section-title"><span>重复间隔</span><button class="text-button icon-button" aria-label="间隔帮助" title="查看间隔说明" @click="openInfo('interval')"><NavIcon name="help"/></button></div>
          <div class="parameter-grid"><label class="field"><span>最小间隔</span><span class="unit-field"><input v-model="intervalMin" type="number" :disabled="inputLocked" min="1" max="60000" step="1"><span>毫秒</span></span></label><label class="field"><span>最大间隔</span><span class="unit-field"><input v-model="intervalMax" type="number" :disabled="inputLocked" min="1" max="60000" step="1"><span>毫秒</span></span></label></div><p v-if="intervalProblem" class="helper overlay-error" role="alert">{{ intervalProblem }}</p><p v-else class="helper">{{ intervalHint }}</p>
          <div class="line"></div><fieldset :disabled="inputLocked"><legend>执行时长</legend><div class="position-options"><label><input v-model="durationMode" type="radio" value="permanent" name="duration">永久</label><label><input v-model="durationMode" type="radio" value="timed" name="duration">限时</label></div><div v-if="durationMode === 'timed'" class="duration-grid"><label class="field">小时<input v-model="durationHours" type="number" min="0" max="99" step="1"></label><label class="field">分钟<input v-model="durationMinutes" type="number" min="0" max="59" step="1"></label><label class="field">秒<input v-model="durationSeconds" type="number" min="0" max="59" step="1"></label></div></fieldset><p v-if="durationProblem" class="helper overlay-error" role="alert">{{ durationProblem }}</p><p v-else class="helper">{{ durationMode === 'permanent' ? '持续执行，直到你手动停止。' : '准备结束后开始计时，到期自动停止。' }}</p>
        </section>
        <section v-else-if="page === 'settings'" class="panel" aria-label="设置"><label class="preference"><span><strong>最小化时保留任务栏图标</strong><small>关闭时仅保留托盘入口</small></span><input :checked="keepTaskbar" :disabled="!initialized || saving || inputLocked" type="checkbox" role="switch" @change="savePreference"></label><div class="line"></div><dl class="settings-details"><div><dt>关闭窗口</dt><dd>询问收起或退出</dd></div><div><dt>本地数据</dt><dd>程序所在文件夹内的 <code>data/</code></dd></div></dl></section>
        <section v-else class="panel about" aria-label="关于"><img :src="appIcon" alt="ClickMaster 图标"><h2>ClickMaster</h2><p>Windows 后台连点工具</p><p>版本 <span data-testid="backend-version">{{ version }}</span></p><p class="helper">便携运行 · 本地配置</p></section>
        <p v-if="inputLocked" class="task-lock">运行期间保留当前设置，停止后可修改。</p>
        <div v-if="criticalPhase" class="execution-error" role="alert"><p>{{ inputStatus.message }}</p><p v-if="inputStatus.down?.error">{{ inputStatus.down.error.reason }}</p><p v-if="inputStatus.up?.error">{{ inputStatus.up.error.reason }}</p></div>
      </div>
      <footer class="runbar"><div class="runtime" aria-live="polite"><strong v-if="!inputStatus.taskId">随时可以开始</strong><strong v-else>{{ progressText }}</strong><span v-if="!inputStatus.taskId">开始前有 3 秒准备时间</span><span v-else title="完整消息投递或 IPC 单帧写入次数，不代表游戏响应">本轮{{ inputStatus.channel === 'taptaploot-ipc' ? '已发送' : '已投递' }} {{ inputStatus.completed }} 次</span></div><div class="actions"><button class="text-button single-action" :disabled="inputLocked || targetBusy || overlayBusy || !bindingStatus.bound || !executionReady" @click="startSingle">执行一次</button><button :disabled="inputLocked || targetBusy || overlayBusy || !bindingStatus.bound || !executionReady || !!intervalProblem || !!durationProblem" class="primary" @click="startRepeat"><NavIcon name="play"/>开始</button><button :disabled="!inputStatus.busy || inputBusy" @click="cancelSingle"><NavIcon name="stop"/>停止<kbd>F8</kbd></button><div class="more-wrap" @focusout="leaveMore" @keydown.esc="moreOpen = false; moreButton?.focus()"><button ref="moreButton" class="text-button icon-button" aria-label="更多操作" title="更多操作" :aria-expanded="moreOpen" aria-controls="more-menu" @click="moreOpen = !moreOpen"><NavIcon name="more"/></button><div v-if="moreOpen" id="more-menu" class="more-menu"><button @click="openInfo('diagnostics')"><NavIcon name="activity"/>诊断信息</button></div></div></div></footer>
    </main>
  </div>
  <dialog ref="infoDialog" class="info-dialog" :class="{ 'diagnostic-dialog': infoKind === 'diagnostics' }" aria-labelledby="info-title" @close="restoreInfoFocus"><div class="dialog-heading"><h2 id="info-title">{{ infoTitle }}</h2><button ref="infoClose" class="text-button icon-button" aria-label="关闭详情" @click="closeInfo"><NavIcon name="close"/></button></div><div class="info-content">
    <template v-if="infoKind === 'details'">
      <p v-if="targetLoaded && !targetLoading" class="helper">匹配 {{ filteredWindows.length }} / {{ windows.length }} 个候选窗口</p>
      <p v-if="bindingStatus.bound" class="helper">当前绑定：{{ displayTitle(bindingStatus.bound) }} · {{ displayStatus(bindingStatus.bound) }}。筛选和刷新不解除有效绑定。</p>
      <p v-if="!includeHidden" class="helper">当前仅列出可见主窗口（含最小化）；需要时可开启后台窗口。无应用窗口的后台进程不会列出。</p>
            <p v-if="inaccessible && !targetLoading" class="helper">{{ inaccessible }} 个候选窗口无法读取必要身份（可能权限不足或已退出），已排除；不会自动提升权限。</p>
            <dl class="window-details"><div><dt>窗口标题</dt><dd>{{ selected ? displayTitle(selected) : (bindingStatus.bound ? displayTitle(bindingStatus.bound) : '—') }}</dd></div><div><dt>进程名称</dt><dd>{{ selected?.exe ?? bindingStatus.bound?.exe ?? '—' }}</dd></div><div><dt>进程 ID</dt><dd>{{ selected?.pid ?? bindingStatus.bound?.pid ?? '—' }}</dd></div></dl>
            <div v-if="bindingStatus.bound && !isIpc" class="receiver-panel"><p class="helper"><strong>输入接收窗口（{{ routeMode(bindingStatus.routing.phase) }}）</strong>：{{ receiverSummary }}</p><p class="helper">{{ bindingStatus.routing.message }}</p><div v-if="bindingStatus.routing.candidates.length" class="form-row"><label class="field grow" for="receiver"><span>选择输入接收窗口</span><select id="receiver" v-model="receiverToken" :disabled="targetBusy || inputLocked"><option value="">请选择接收窗口</option><option v-for="receiver in bindingStatus.routing.candidates" :key="receiver.token" :value="receiver.token">{{ receiverLabel(receiver) }}</option></select></label><button :disabled="!receiverToken || targetBusy || inputLocked" @click="selectReceiver">确认接收窗口</button></div></div>

    </template>
    <template v-else-if="infoKind === 'diagnostics'"><div class="diagnostic-content">
      <p><strong>兼容与执行</strong>：{{ channelStatus.reason }}</p>
      <p>{{ progressText }} · 已完成 {{ inputStatus.completed }} 次</p>
      <p v-if="inputError">{{ inputError }}</p><p v-if="targetError">{{ targetError }}</p><p v-if="bindingStatus.reason">{{ bindingStatus.reason }}</p>
      <p v-if="isIpc">向已核验的 TapTapLoot 实例重复发送固定指令，不使用键鼠或坐标。游戏可在前台或后台，请保持非最小化。可正常操作游戏，人工与自动效果可能叠加；客户端读取确认不等于玩法响应。</p>
      <p>单次与重复均先准备3秒；窗口消息完成按下与释放，IPC 每轮完整写入一帧即记为已发送，读取诊断不阻断后续执行。收起到托盘继续执行，F8 / 托盘 / 停止按钮可结束。限时从准备结束后开始，包含执行与等待；到期不再开始新操作。</p>
        <div class="performance-control"><strong>性能采集</strong><p aria-live="polite">{{ performanceView.message }}<span v-if="performanceView.active"> · {{ performanceView.elapsedSeconds }} / 60 秒</span></p><button :disabled="performanceBusy || performanceView.active" @click="capturePerformance(true)">开始性能采集</button> <button :disabled="performanceBusy || !performanceView.active" @click="capturePerformance(false)">结束采集</button><p v-if="performanceView.fileStem">日志：<code>data/logs/{{ performanceView.fileStem }}.jsonl</code> 和同名 <code>.txt</code></p><p class="helper">手动开启，每秒采样，60秒自动结束；不启动或停止连点。自动对照待命/执行、显示/收起。WebView环境可能共享、不含crashpad；缺失不等于0，工作集可能重叠。</p></div>
        <p>{{ channelLabel }} · {{ isIpc ? '原绑定进程专用客户端' : `接收路由（${routeMode(bindingStatus.routing.phase)}）：${receiverSummary}` }}</p>
        <p>{{ inputStatus.summary }}{{ inputStatus.summary ? '：' : '' }}{{ inputStatus.message }}</p>
        <p v-if="inputStatus.receiver">本次接收：{{ receiverLabel(inputStatus.receiver) }}</p>
        <p v-if="inputStatus.down">{{ receiptText('按下', inputStatus.down) }}</p><p v-if="inputStatus.up">{{ receiptText('释放', inputStatus.up) }}</p>
        <p v-if="inputStatus.ipc">连接：{{ inputStatus.ipc.connected ? '已核验' : '未确认' }} · 单帧写入：{{ inputStatus.ipc.written ? '已完成' : '未确认' }} · 读取诊断：{{ ipcReadText(inputStatus.ipc) }}</p>
        <p v-if="inputStatus.ipc?.failure">{{ ipcFailureText(inputStatus.ipc) }}</p>
        <p>次数为完整消息投递或 IPC 单帧写入次数，不是游戏计数；已发出的操作无法撤回。不自动重试或回退全局输入。</p>
        <p v-if="isIpc">IPC 每轮固定一帧；重复运行复用一个已核验连接和自有worker，连接最多20秒、单帧写入最多2秒。重复时不等待读取确认；极短间隔可能积压并因写超时停止。已发送帧无法撤回，游戏可能在停止后继续处理。单次仍短时观察读取结果并清理。</p>
        <template v-if="!isIpc && isMouse"><div class="overlay-control"><label class="range-option"><input type="checkbox" :checked="overlayStatus.enabled" :disabled="!bindingStatus.bound || overlayBusy || targetBusy || inputLocked" @change="updateOverlay(($event.target as HTMLInputElement).checked)">显示坐标预览（实验）</label><p class="helper" role="status">{{ overlayStatus.message }}</p><p v-if="overlayError" class="helper overlay-error" role="alert">{{ overlayError }}</p><p class="helper">已知预览未显示问题待处理；单次输入不依赖预览。客户区坐标使用当前像素。</p></div></template>
    </div></template>
    <template v-else-if="infoKind === 'interval'"><p>可设置 1～60000 毫秒，默认 2000 毫秒。相同数值为固定间隔，不同数值为随机区间。每次完整操作结束后再等待，实际速度受系统调度与传输耗时影响。1 毫秒仅用于边界测试，不承诺 1kHz。</p></template>
    <template v-else>
      <p v-if="page === 'target'">搜索并选择目标应用，再点击绑定窗口。绑定不会发送输入。后台窗口可通过“显示后台窗口”查找；无窗口的后台进程无法绑定。更多身份与接收设置见“窗口详情”。</p>
      <template v-else-if="page === 'input'"><p>窗口消息模式支持鼠标左键、右键和单个按键，每次完整按下并释放。窗口内容区不含标题栏与边框；选择中心时无需填写坐标。</p><p>指定坐标以绑定窗口客户区当前像素为准；发送前转换到接收窗口客户区，超出接收区域则不发送。窗口移动不改变内容区坐标。</p><p>TapTapLoot 专用模式仅在兼容时可选，使用固定指令，无需选择按键或坐标。连接与读取结果可在诊断信息中查看。</p></template>
      <p v-else-if="page === 'execution'">设置固定或随机间隔，选择永久或限时执行。单次与重复执行前均有 3 秒准备时间。永久模式持续到手动停止或保护条件触发；限时从准备结束后计时，包含执行与等待。F8、停止按钮或托盘均可停止。</p>
      <p v-else-if="page === 'settings'">最小化默认收起到托盘，可选择保留任务栏图标。关闭窗口时会询问收起或退出。本地数据保存在程序同级 data/ 文件夹。</p>
      <p v-else>ClickMaster 是便携式 Windows 后台连点工具。完整消息投递或 IPC 单帧写入次数不代表游戏响应。必要日志、报错与性能采集可从底部“更多操作 → 诊断信息”查看。</p>
    </template>
  </div><div class="info-dialog-actions"><button @click="closeInfo">知道了</button></div></dialog>
  <dialog ref="closeDialog" class="close-dialog" aria-labelledby="close-title" aria-describedby="close-description" @cancel.prevent="resolveClose('cancel')"><h2 id="close-title">关闭 ClickMaster</h2><p id="close-description">你希望收起窗口，还是退出程序？收起遵循任务栏偏好，输入草稿会保留。</p><p v-if="closeError" class="dialog-error" role="alert">{{ closeError }}</p><div class="close-actions"><button ref="cancelButton" :disabled="closeBusy" @click="resolveClose('cancel')">取消</button><button :disabled="closeBusy" @click="resolveClose('tuck')">收起到托盘</button><button class="primary" :disabled="closeBusy" @click="resolveClose('exit')">退出程序</button></div></dialog>
</template>
