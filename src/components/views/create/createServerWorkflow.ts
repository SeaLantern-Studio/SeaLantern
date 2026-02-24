import type { StartupCandidate } from "@components/views/create/startupTypes";
import { sortStartupCandidates } from "@components/views/create/startupUtils";
import { i18n } from "@language";

export function appendCustomCandidate(candidates: StartupCandidate[]): StartupCandidate[] {
  // 将“自定义命令”固定追加到列表末尾，保证扫描结果和手动输入同时可选。
  const custom: StartupCandidate = {
    id: "custom-command",
    mode: "custom",
    label: i18n.t("create.startup_candidate_custom"),
    detail: i18n.t("create.startup_candidate_custom_desc"),
    path: "",
    recommended: 9,
  };

  return [...sortStartupCandidates(candidates), custom];
}
