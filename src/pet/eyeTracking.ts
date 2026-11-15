export interface Point {
  x: number;
  y: number;
}

export interface EyeOffset {
  x: number;
  y: number;
  rotation: number;
  intensity: number;
  tracking: boolean;
  distance: number;
}

export interface EyeSpringState {
  x: number;
  y: number;
  velocityX: number;
  velocityY: number;
  rotation: number;
  angularVelocity: number;
  intensity: number;
  intensityVelocity: number;
}

export interface EyeMotion {
  x: number;
  y: number;
  rotation: number;
  intensity: number;
}

export const EYE_TRACK_RADIUS = 720;
export const EYE_MAX_OFFSET = 7;
const EYE_DEAD_ZONE = 4;
const MAX_ROTATION = 6;

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

/**
 * 计算眼睛相对眼眶的目标偏移。
 *
 * 参考了开源项目 jqEye（MPL-2.0）中「沿鼠标方向在眼眶边界内取点」的思路，
 * 这里改成适合海晶灯竖眼的远距离向量追踪：范围扩大到 720px，远处使用缓动曲线，
 * 同时返回轻微旋转和注视强度，让眼睛不是机械地平移。
 */
export function calculateEyeOffset(
  cursor: Point,
  petCenter: Point,
  radius = EYE_TRACK_RADIUS,
  maxOffset = EYE_MAX_OFFSET,
): EyeOffset {
  const dx = cursor.x - petCenter.x;
  const dy = cursor.y - petCenter.y;
  const distance = Math.hypot(dx, dy);

  if (!Number.isFinite(distance) || distance > radius) {
    return { x: 0, y: 0, rotation: 0, intensity: 0, tracking: false, distance };
  }
  if (distance <= EYE_DEAD_ZONE) {
    return { x: 0, y: 0, rotation: 0, intensity: 1, tracking: true, distance };
  }

  const normalizedDistance = clamp(distance / radius, 0, 1);
  // 0.62 次方让中远距离也有明显的跟随感，而不是只在桌宠身边才动。
  const strength = clamp((distance - EYE_DEAD_ZONE) / (radius - EYE_DEAD_ZONE), 0, 1) ** 0.62;
  const directionX = dx / distance;
  const directionY = dy / distance;
  const intensity = 1 - normalizedDistance;

  return {
    x: directionX * maxOffset * strength,
    y: directionY * maxOffset * 0.78 * strength,
    rotation: directionX * MAX_ROTATION * strength,
    intensity,
    tracking: true,
    distance,
  };
}

export function createEyeSpringState(): EyeSpringState {
  return {
    x: 0,
    y: 0,
    velocityX: 0,
    velocityY: 0,
    rotation: 0,
    angularVelocity: 0,
    intensity: 0,
    intensityVelocity: 0,
  };
}

/**
 * 60fps 阻尼弹簧。鼠标目标位置由低频 IPC 更新，眼睛本体仍然每帧平滑运动，
 * 避免 Tauri 全局鼠标轮询间隔带来的逐格跳动。
 */
export function stepEyeSpring(
  state: EyeSpringState,
  target: EyeOffset,
  deltaSeconds: number,
): EyeMotion {
  const dt = clamp(Number.isFinite(deltaSeconds) ? deltaSeconds : 0, 0, 0.033);
  if (dt === 0) {
    return {
      x: state.x,
      y: state.y,
      rotation: state.rotation,
      intensity: state.intensity,
    };
  }

  state.velocityX += (-310 * (state.x - target.x) - 25 * state.velocityX) * dt;
  state.velocityY += (-330 * (state.y - target.y) - 26 * state.velocityY) * dt;
  state.angularVelocity +=
    (-260 * (state.rotation - target.rotation) - 22 * state.angularVelocity) * dt;
  state.intensityVelocity +=
    (-150 * (state.intensity - target.intensity) - 18 * state.intensityVelocity) * dt;

  state.x += state.velocityX * dt;
  state.y += state.velocityY * dt;
  state.rotation += state.angularVelocity * dt;
  state.intensity += state.intensityVelocity * dt;

  state.x = clamp(state.x, -10, 10);
  state.y = clamp(state.y, -9, 9);
  state.rotation = clamp(state.rotation, -8, 8);
  state.intensity = clamp(state.intensity, 0, 1);

  return {
    x: state.x,
    y: state.y,
    rotation: state.rotation,
    intensity: state.intensity,
  };
}
