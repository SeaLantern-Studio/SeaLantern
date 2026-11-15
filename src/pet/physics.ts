export interface PetSpringState {
  offsetX: number;
  offsetY: number;
  velocityX: number;
  velocityY: number;
  scaleX: number;
  scaleY: number;
  scaleVelocityX: number;
  scaleVelocityY: number;
  rotation: number;
  angularVelocity: number;
}

export interface PetSpringTransform {
  x: number;
  y: number;
  scaleX: number;
  scaleY: number;
  rotation: number;
}

const OFFSET_LIMIT = 34;
const ROTATION_LIMIT = 9;

export const JELLY_SPEED_CYCLES_PER_SECOND = 1.1;
export const JELLY_STRENGTH = 0.38;

export function resetPetLateralMotion(state: PetSpringState): void {
  state.offsetX = 0;
  state.velocityX = 0;
  state.rotation = 0;
  state.angularVelocity = 0;
}

export function resetPetSpringState(state: PetSpringState): void {
  Object.assign(state, createPetSpringState());
}

/**
 * 参考项目 DesktopJelly 的原始 Motion.Scale：
 * 底边固定，只做纵向拉长/压扁和反向水平形变，没有位移、摇摆或旋转。
 */
export function calculateJellyTransform(
  elapsedSeconds: number,
  strength = JELLY_STRENGTH,
): { scaleX: number; scaleY: number; wave: number } {
  const wave = Math.cos(elapsedSeconds * Math.PI * 2 * JELLY_SPEED_CYCLES_PER_SECOND);
  return {
    scaleX: 1 - strength * 0.72 * wave,
    scaleY: 1 + strength * wave,
    wave,
  };
}

export function createPetSpringState(): PetSpringState {
  return {
    offsetX: 0,
    offsetY: 0,
    velocityX: 0,
    velocityY: 0,
    scaleX: 1,
    scaleY: 1,
    scaleVelocityX: 0,
    scaleVelocityY: 0,
    rotation: 0,
    angularVelocity: 0,
  };
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

/**
 * 给桌宠施加一次冲量。
 *
 * 纵向冲量负责“弹起来”，横向冲量和角速度负责被打一下之后的小幅摇摆；
 * 两个缩放方向反向冲量形成挤压/拉伸，弹簧会在下一帧把它拉回 1:1。
 */
export function nudgePetSpring(state: PetSpringState, strength = 1, direction = 1): PetSpringState {
  const amount = clamp(strength, 0, 3);
  state.velocityY -= 225 * amount;
  state.velocityX += direction * 88 * amount;
  state.angularVelocity += direction * 12 * amount;
  return state;
}

/**
 * 用阻尼弹簧积分桌宠位移、旋转和挤压形变。
 *
 * 不使用 CSS 关键帧，保证鼠标拖拽、点击、表情切换和通透模式鼓点
 * 都能向同一套物理状态施加冲量，动作不会互相打架。
 */
export function stepPetSpring(
  state: PetSpringState,
  deltaSeconds: number,
  elapsedSeconds: number,
  dancing = false,
): PetSpringTransform {
  const dt = clamp(Number.isFinite(deltaSeconds) ? deltaSeconds : 0, 0, 0.033);
  if (dt === 0) {
    return {
      x: state.offsetX,
      y: state.offsetY,
      scaleX: state.scaleX,
      scaleY: state.scaleY,
      rotation: state.rotation,
    };
  }

  // 通透模式遵循原版 Squash & Stretch：只做纵向弹跳和体积形变，
  // 不产生左右摇摆、横向惯性或旋转。
  const idleTargetY = dancing ? 0 : Math.sin(elapsedSeconds * 1.7) * 1.8;
  const idleTargetX = 0;
  const idleTargetRotation = 0;

  // 位移弹簧：偏硬、阻尼略小，松开后会有肉眼可见的余振。
  state.velocityY += (-214 * (state.offsetY - idleTargetY) - 13.5 * state.velocityY) * dt;
  state.velocityX += (-176 * (state.offsetX - idleTargetX) - 12.2 * state.velocityX) * dt;
  state.offsetY += state.velocityY * dt;
  state.offsetX += state.velocityX * dt;

  // 通透模式的「变长 / 变扁」：向上冲时纵向拉长、横向收窄；
  // 下落与落地时反过来压扁、横向撑宽。普通状态保留轻微形变。
  const motionStretch = clamp(-state.velocityY / 260, -1, 1);
  const deformation = motionStretch * (dancing ? 0.35 : 0.12);
  const targetScaleY = 1 + deformation;
  const targetScaleX = 1 - deformation * 0.82;
  state.scaleVelocityX += (-620 * (state.scaleX - targetScaleX) - 18 * state.scaleVelocityX) * dt;
  state.scaleVelocityY += (-620 * (state.scaleY - targetScaleY) - 18 * state.scaleVelocityY) * dt;
  state.scaleX += state.scaleVelocityX * dt;
  state.scaleY += state.scaleVelocityY * dt;

  state.angularVelocity +=
    (-188 * (state.rotation - idleTargetRotation) - 14.5 * state.angularVelocity) * dt;
  state.rotation += state.angularVelocity * dt;

  state.offsetX = clamp(state.offsetX, -OFFSET_LIMIT, OFFSET_LIMIT);
  state.offsetY = clamp(state.offsetY, -OFFSET_LIMIT, OFFSET_LIMIT);
  state.scaleX = clamp(state.scaleX, 0.64, 1.36);
  state.scaleY = clamp(state.scaleY, 0.64, 1.36);
  state.rotation = clamp(state.rotation, -ROTATION_LIMIT, ROTATION_LIMIT);

  return {
    x: state.offsetX,
    y: state.offsetY,
    scaleX: state.scaleX,
    scaleY: state.scaleY,
    rotation: state.rotation,
  };
}
