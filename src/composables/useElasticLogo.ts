/**
 * 海景灯图标彩蛋：长按 3 秒激活拖动，松手后弹力绳弹飞效果
 * 弹力绳使用 SVG 贝塞尔曲线渲染，物理模拟基于弹簧 + 阻尼
 */
import { ref, onUnmounted } from "vue";

interface Anchor {
  x: number;
  y: number;
}

interface Velocity {
  vx: number;
  vy: number;
}

const HOLD_THRESHOLD = 3000; // 长按激活阈值（毫秒）
const SPRING_K = 0.12; // 弹簧劲度系数
const DAMPING = 0.86; // 阻尼系数
const MAX_VELOCITY = 60; // 限制最大速度，避免飞太快

/** 获取元素相对视口的锚点（中心点） */
function getAnchor(el: HTMLElement): Anchor {
  const rect = el.getBoundingClientRect();
  return { x: rect.left + rect.width / 2, y: rect.top + rect.height / 2 };
}

export function useElasticLogo() {
  const isDragging = ref(false);
  const isArmed = ref(false); // 已激活、等待拖动
  const isAnimating = ref(false); // 弹飞动画进行中
  const holdProgress = ref(0); // 长按进度 0-1，用于环形进度提示

  const position = ref<Anchor>({ x: 0, y: 0 });
  const anchor = ref<Anchor>({ x: 0, y: 0 });
  const velocity = ref<Velocity>({ vx: 0, vy: 0 });

  let holdTimer: ReturnType<typeof setTimeout> | null = null;
  let progressRafId: number | null = null;
  let rafId: number | null = null;
  let lastPointerX = 0;
  let lastPointerY = 0;
  let lastTime = 0;

  /** 开始长按检测 */
  function startHold(e: MouseEvent | TouchEvent, el: HTMLElement) {
    if (isArmed.value || isAnimating.value) return;
    anchor.value = getAnchor(el);

    const point = "touches" in e ? e.touches[0] : (e as MouseEvent);
    lastPointerX = point.clientX;
    lastPointerY = point.clientY;
    lastTime = performance.now();

    holdProgress.value = 0;
    const start = performance.now();

    // 用 rAF 替代 setInterval(30ms),自然适配刷新率并避免后台标签页持续触发
    const tickProgress = () => {
      const elapsed = performance.now() - start;
      holdProgress.value = Math.min(1, elapsed / HOLD_THRESHOLD);
      if (elapsed < HOLD_THRESHOLD) {
        progressRafId = requestAnimationFrame(tickProgress);
      }
    };
    progressRafId = requestAnimationFrame(tickProgress);

    holdTimer = setTimeout(() => {
      isArmed.value = true;
      holdProgress.value = 0;
      if (progressRafId !== null) {
        cancelAnimationFrame(progressRafId);
        progressRafId = null;
      }
    }, HOLD_THRESHOLD);
  }

  /** 取消长按 */
  function cancelHold() {
    if (holdTimer) {
      clearTimeout(holdTimer);
      holdTimer = null;
    }
    if (progressRafId !== null) {
      cancelAnimationFrame(progressRafId);
      progressRafId = null;
    }
    holdProgress.value = 0;
  }

  /** 激活后开始拖动 */
  function startDrag(e: MouseEvent | TouchEvent) {
    if (!isArmed.value || isAnimating.value) return;
    const point = "touches" in e ? e.touches[0] : (e as MouseEvent);
    lastPointerX = point.clientX;
    lastPointerY = point.clientY;
    lastTime = performance.now();
    isDragging.value = true;
    position.value = { x: point.clientX, y: point.clientY };
    velocity.value = { vx: 0, vy: 0 };
  }

  /** 拖动中 */
  function moveDrag(e: MouseEvent | TouchEvent) {
    if (!isDragging.value) return;
    const point = "touches" in e ? e.touches[0] : (e as MouseEvent);
    const now = performance.now();
    const dt = Math.max(1, now - lastTime);

    // 估算瞬时速度（像素/帧，按 16ms 标准化）
    const dx = point.clientX - lastPointerX;
    const dy = point.clientY - lastPointerY;
    velocity.value = {
      vx: (dx / dt) * 16,
      vy: (dy / dt) * 16,
    };

    lastPointerX = point.clientX;
    lastPointerY = point.clientY;
    lastTime = now;
    position.value = { x: point.clientX, y: point.clientY };
  }

  /** 松手，开始弹飞动画 */
  function releaseDrag() {
    if (!isDragging.value) return;
    isDragging.value = false;
    isAnimating.value = true;

    // 限制初始速度
    const v = velocity.value;
    const speed = Math.hypot(v.vx, v.vy);
    if (speed > MAX_VELOCITY) {
      const scale = MAX_VELOCITY / speed;
      velocity.value = { vx: v.vx * scale, vy: v.vy * scale };
    }

    runAnimation();
  }

  /** 弹簧物理动画循环 */
  function runAnimation() {
    const step = () => {
      if (!isAnimating.value) return;

      const dx = anchor.value.x - position.value.x;
      const dy = anchor.value.y - position.value.y;

      // 弹簧力：F = -k * x
      const ax = SPRING_K * dx;
      const ay = SPRING_K * dy;

      velocity.value.vx = (velocity.value.vx + ax) * DAMPING;
      velocity.value.vy = (velocity.value.vy + ay) * DAMPING;

      position.value.x += velocity.value.vx;
      position.value.y += velocity.value.vy;

      // 速度足够小且接近锚点时停止
      const speed = Math.hypot(velocity.value.vx, velocity.value.vy);
      const dist = Math.hypot(dx, dy);
      if (speed < 0.1 && dist < 0.5) {
        position.value = { ...anchor.value };
        velocity.value = { vx: 0, vy: 0 };
        isAnimating.value = false;
        isArmed.value = false;
        rafId = null;
        return;
      }

      rafId = requestAnimationFrame(step);
    };
    rafId = requestAnimationFrame(step);
  }

  /** 生成弹力绳贝塞尔路径 */
  function elasticPath(): string {
    const a = anchor.value;
    const p = position.value;
    const dx = p.x - a.x;
    const dy = p.y - a.y;
    const dist = Math.hypot(dx, dy);
    // 拖得越远，绳子越细且有弧度
    const sag = Math.min(40, dist * 0.15);
    // 垂直方向偏移产生自然下垂
    const cx = (a.x + p.x) / 2;
    const cy = (a.y + p.y) / 2 + sag;
    return `M ${a.x} ${a.y} Q ${cx} ${cy} ${p.x} ${p.y}`;
  }

  /** 绳子张力颜色：越远越红 */
  function elasticColor(): string {
    const dist = Math.hypot(position.value.x - anchor.value.x, position.value.y - anchor.value.y);
    // 0px → 蓝绿，300px+ → 红
    const t = Math.min(1, dist / 300);
    const r = Math.round(80 + t * 175);
    const g = Math.round(180 - t * 140);
    const b = Math.round(200 - t * 160);
    return `rgb(${r}, ${g}, ${b})`;
  }

  /** 绳子粗细：越远越细 */
  function elasticWidth(): number {
    const dist = Math.hypot(position.value.x - anchor.value.x, position.value.y - anchor.value.y);
    return Math.max(1.2, 4 - dist / 100);
  }

  onUnmounted(() => {
    cancelHold();
    if (rafId) cancelAnimationFrame(rafId);
    rafId = null;
  });

  return {
    isDragging,
    isArmed,
    isAnimating,
    holdProgress,
    position,
    anchor,
    startHold,
    cancelHold,
    startDrag,
    moveDrag,
    releaseDrag,
    elasticPath,
    elasticColor,
    elasticWidth,
  };
}
