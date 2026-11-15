import assert from "node:assert/strict";
import test from "node:test";
import {
  EYE_MAX_OFFSET,
  EYE_TRACK_RADIUS,
  calculateEyeOffset,
  createEyeSpringState,
  stepEyeSpring,
} from "../src/pet/eyeTracking.ts";

test("桌宠眼睛跟随范围扩大到 720px，范围外归位", () => {
  assert.equal(EYE_TRACK_RADIUS, 720);
  const center = { x: 500, y: 500 };
  const nearby = calculateEyeOffset({ x: 1100, y: 500 }, center);
  const outside = calculateEyeOffset({ x: 1300, y: 500 }, center);

  assert.equal(nearby.tracking, true);
  assert.ok(Math.abs(nearby.x) > EYE_MAX_OFFSET * 0.8);
  assert.equal(outside.tracking, false);
  assert.equal(outside.x, 0);
  assert.equal(outside.y, 0);
});

test("眼睛偏移按距离缓动，并带有轻微的注视旋转", () => {
  const center = { x: 0, y: 0 };
  const mid = calculateEyeOffset({ x: 360, y: 0 }, center);
  const far = calculateEyeOffset({ x: 700, y: 0 }, center);

  assert.ok(far.x > mid.x, "更远的鼠标应该带来更大的偏移");
  assert.ok(far.x <= EYE_MAX_OFFSET + 0.001, "偏移不能超过眼眶范围");
  assert.ok(far.rotation > 0, "向右看时应该有轻微旋转");
  assert.ok(far.intensity >= 0 && far.intensity <= 1);
});

test("鼠标落在桌宠中心时保持眼睛归位", () => {
  const result = calculateEyeOffset({ x: 10, y: 10 }, { x: 8, y: 8 });
  assert.equal(result.x, 0);
  assert.equal(result.y, 0);
  assert.equal(result.tracking, true);
});

test("眼睛弹簧会平滑追上目标，并在目标归零后回到中心", () => {
  const center = { x: 0, y: 0 };
  const target = calculateEyeOffset({ x: 600, y: 200 }, center);
  const state = createEyeSpringState();
  let motion = stepEyeSpring(state, target, 1 / 60);
  for (let i = 0; i < 30; i += 1) motion = stepEyeSpring(state, target, 1 / 60);

  assert.ok(motion.x > target.x * 0.7, "弹簧应快速接近目标");
  const rest = calculateEyeOffset(center, center);
  for (let i = 0; i < 120; i += 1) motion = stepEyeSpring(state, rest, 1 / 60);
  assert.ok(Math.abs(motion.x) < 0.05, `眼睛应归位，实际 ${motion.x}`);
  assert.ok(Math.abs(motion.y) < 0.05, `眼睛应归位，实际 ${motion.y}`);
});
