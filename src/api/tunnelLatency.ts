/**
 * 加入方隧道的实时延迟共享值。
 *
 * 状态轮询有 5s，用它驱动趋势图会让曲线明显滞后；sculk 的 `path_changed`
 * 事件按 1s 节流，因此这里保存最新值，由趋势图按 1s 采样读取。
 * 刻意不用 `ref`：采样只需读数值，不需要触发组件重渲染。
 */
let liveRttMs: number | null = null;

/** 写入最新 RTT；传 `null` 表示当前没有可用的延迟值（如隧道停止）。 */
export function setLiveRtt(value: number | null) {
  liveRttMs = value != null && Number.isFinite(value) && value >= 0 ? value : null;
}

export function getLiveRtt(): number | null {
  return liveRttMs;
}
