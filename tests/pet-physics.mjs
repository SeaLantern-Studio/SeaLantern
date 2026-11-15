import assert from "node:assert/strict";
import test from "node:test";
import {
  calculateJellyTransform,
  createPetSpringState,
  nudgePetSpring,
  resetPetLateralMotion,
  stepPetSpring,
} from "../src/pet/physics.ts";

const simulate = (state, seconds, dancing = false) => {
  const steps = Math.round(seconds / (1 / 120));
  for (let i = 0; i < steps; i += 1) {
    stepPetSpring(state, 1 / 120, i / 120, dancing);
  }
};

test("桌宠受到冲量后会向上弹起，并先变长再变扁", () => {
  const state = createPetSpringState();
  nudgePetSpring(state, 1.3, 1);
  let maxScaleY = 1;
  let minScaleY = 1;
  for (let i = 0; i < 120; i += 1) {
    stepPetSpring(state, 1 / 120, i / 120, true);
    maxScaleY = Math.max(maxScaleY, state.scaleY);
    minScaleY = Math.min(minScaleY, state.scaleY);
  }

  assert.ok(state.offsetX !== 0, "横向应产生摇摆");
  assert.ok(maxScaleY > 1.1, `向上跳时应明显拉长，实际 ${maxScaleY}`);
  assert.ok(minScaleY < 0.9, `下落时变扁，实际 ${minScaleY}`);
});

test("阻尼弹簧最终回到静止姿态", () => {
  const state = createPetSpringState();
  nudgePetSpring(state, 1.4, -1);
  simulate(state, 4);
  const idleTargetY = Math.sin(4 * 1.7) * 1.8;

  assert.ok(Math.abs(state.offsetX) < 0.35, `x 应归零，实际 ${state.offsetX}`);
  assert.ok(
    Math.abs(state.offsetY - idleTargetY) < 0.35,
    `y 应贴合呼吸目标，实际 ${state.offsetY}`,
  );
  assert.ok(Math.abs(state.scaleX - 1) < 0.015, `scaleX 应恢复，实际 ${state.scaleX}`);
  assert.ok(Math.abs(state.scaleY - 1) < 0.015, `scaleY 应恢复，实际 ${state.scaleY}`);
  assert.ok(Math.abs(state.rotation) < 0.35, `旋转应归零，实际 ${state.rotation}`);
});

test("通透模式只做纵向 Squash & Stretch，不产生左右晃动和旋转", () => {
  const state = createPetSpringState();
  let maxLateral = 0;
  let maxRotation = 0;
  for (let i = 0; i < 360; i += 1) {
    if (i % 28 === 0) nudgePetSpring(state, 1.3, 0);
    stepPetSpring(state, 1 / 120, i / 120, true);
    maxLateral = Math.max(maxLateral, Math.abs(state.offsetX));
    maxRotation = Math.max(maxRotation, Math.abs(state.rotation));
  }
  assert.ok(maxLateral < 0.001, `不应横向晃动，实际 ${maxLateral}`);
  assert.ok(maxRotation < 0.001, `不应旋转，实际 ${maxRotation}`);
});

test("退出通透模式前可以清除遗留的横向速度和旋转", () => {
  const state = createPetSpringState();
  nudgePetSpring(state, 1.4, 1);
  for (let i = 0; i < 10; i += 1) stepPetSpring(state, 1 / 120, i / 120, true);
  resetPetLateralMotion(state);
  assert.equal(state.offsetX, 0);
  assert.equal(state.velocityX, 0);
  assert.equal(state.rotation, 0);
  assert.equal(state.angularVelocity, 0);
});

test("原版 DesktopJelly 周期为 1.1Hz，且只改变宽高比例", () => {
  const start = calculateJellyTransform(0);
  const halfCycle = calculateJellyTransform(1 / (1.1 * 2));
  assert.ok(Math.abs(start.scaleY - 1.38) < 0.0001, `起始纵向拉伸 ${start.scaleY}`);
  assert.ok(Math.abs(start.scaleX - (1 - 0.38 * 0.72)) < 0.0001, `起始横向压缩 ${start.scaleX}`);
  assert.ok(Math.abs(halfCycle.scaleY - 0.62) < 0.0001, `半周期纵向压扁 ${halfCycle.scaleY}`);
  assert.ok(
    Math.abs(halfCycle.scaleX - (1 + 0.38 * 0.72)) < 0.0001,
    `半周期横向撑宽 ${halfCycle.scaleX}`,
  );
});
