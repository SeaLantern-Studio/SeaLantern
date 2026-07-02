import {
  computed,
  inject,
  onBeforeUnmount,
  provide,
  shallowRef,
  watchEffect,
  type InjectionKey,
  type Ref,
  type ShallowRef,
  type VNode,
} from "vue";

export interface NextProtectedShellRenderState {
  railLocked: boolean;
  shellHeaderMode: "title" | "hidden";
}

export interface NextProtectedShellController {
  renderState: Readonly<ShallowRef<NextProtectedShellRenderState>>;
  headerPrimaryActions: Readonly<ShallowRef<(() => VNode[]) | null>>;
  setRegistration: (
    token: symbol,
    registration: {
      renderState: NextProtectedShellRenderState;
      headerPrimaryActions: (() => VNode[]) | null;
    },
  ) => void;
  clearRegistration: (token: symbol) => void;
}

const NEXT_PROTECTED_SHELL_CONTROLLER_KEY: InjectionKey<NextProtectedShellController> =
  Symbol("next-protected-shell-controller");

const DEFAULT_RENDER_STATE: NextProtectedShellRenderState = {
  railLocked: false,
  shellHeaderMode: "title",
};

export function provideNextProtectedShellController(): NextProtectedShellController {
  const renderState = shallowRef<NextProtectedShellRenderState>(DEFAULT_RENDER_STATE);
  const headerPrimaryActions = shallowRef<(() => VNode[]) | null>(null);
  const activeRegistrationToken = shallowRef<symbol | null>(null);

  function setRegistration(
    token: symbol,
    registration: {
      renderState: NextProtectedShellRenderState;
      headerPrimaryActions: (() => VNode[]) | null;
    },
  ): void {
    activeRegistrationToken.value = token;
    renderState.value = registration.renderState;
    headerPrimaryActions.value = registration.headerPrimaryActions;
  }

  function clearRegistration(token: symbol): void {
    if (activeRegistrationToken.value !== token) {
      return;
    }

    queueMicrotask(() => {
      if (activeRegistrationToken.value !== token) {
        return;
      }

      activeRegistrationToken.value = null;
      renderState.value = DEFAULT_RENDER_STATE;
      headerPrimaryActions.value = null;
    });
  }

  const controller: NextProtectedShellController = {
    renderState,
    headerPrimaryActions,
    setRegistration,
    clearRegistration,
  };

  provide(NEXT_PROTECTED_SHELL_CONTROLLER_KEY, controller);

  return controller;
}

function useNextProtectedShellController(): NextProtectedShellController {
  const controller = inject(NEXT_PROTECTED_SHELL_CONTROLLER_KEY, null);

  if (!controller) {
    throw new Error("Next protected shell controller is not available in the current route tree.");
  }

  return controller;
}

export function useRegisterNextProtectedShell(input: {
  railLocked?: Ref<boolean>;
  shellHeaderMode?: Ref<"title" | "hidden">;
  headerPrimaryActions?: Ref<(() => VNode[]) | null>;
} = {}): void {
  const controller = useNextProtectedShellController();
  const registrationToken = Symbol("next-protected-shell-registration");

  const railLocked = computed(() => input.railLocked?.value ?? false);
  const shellHeaderMode = computed(() => input.shellHeaderMode?.value ?? "title");
  const headerPrimaryActions = computed(() => input.headerPrimaryActions?.value ?? null);

  watchEffect(() => {
    controller.setRegistration(registrationToken, {
      renderState: {
        railLocked: railLocked.value,
        shellHeaderMode: shellHeaderMode.value,
      },
      headerPrimaryActions: headerPrimaryActions.value,
    });
  });

  onBeforeUnmount(() => {
    controller.clearRegistration(registrationToken);
  });
}
