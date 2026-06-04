import { usePluginsViewBindings } from "@components/views/plugins/usePluginsViewBindings";
import { usePluginsViewState } from "@components/views/plugins/usePluginsViewState";

export function usePluginsViewModel() {
  const state = usePluginsViewState();
  const bindings = usePluginsViewBindings({ state });

  return {
    ...state,
    ...bindings,
  };
}
