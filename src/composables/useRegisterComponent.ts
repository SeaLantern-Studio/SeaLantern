import { onActivated, onDeactivated, onMounted, onUnmounted } from "vue";
import { useComponentRegistry } from "./useComponentRegistry";
import type { ComponentHandle } from "./useComponentRegistry";

export function useRegisterComponent(id: string, handle: ComponentHandle) {
  const reg = useComponentRegistry();
  let registered = false;

  function doRegister() {
    if (registered) return;
    registered = true;
    reg.register(id, handle);
  }

  function doUnregister() {
    if (!registered) return;
    registered = false;
    reg.unregister(id);
  }

  onMounted(doRegister);
  onActivated(doRegister);

  onUnmounted(doUnregister);
  onDeactivated(doUnregister);
}
