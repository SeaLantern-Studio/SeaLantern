export type PetExpression =
  | "neutral"
  | "happy"
  | "angry"
  | "sleepy"
  | "surprised"
  | "heart"
  | "wink";

export interface PetExpressionOption {
  id: PetExpression;
  label: string;
  hint: string;
}

/** 全部表情都只改变眼睛的形状/角度，不替换海晶灯本体贴图。 */
export const PET_EXPRESSIONS: PetExpressionOption[] = [
  { id: "neutral", label: "平常心", hint: "蓝色竖线" },
  { id: "happy", label: "开心", hint: "^ ^" },
  { id: "angry", label: "气鼓鼓", hint: "斜眼" },
  { id: "sleepy", label: "困困", hint: "半闭眼" },
  { id: "surprised", label: "吃惊", hint: "竖线放大" },
  { id: "heart", label: "心动", hint: "蓝色爱心" },
  { id: "wink", label: "眨眼", hint: "单眼微笑" },
];
