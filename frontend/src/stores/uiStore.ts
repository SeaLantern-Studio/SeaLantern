import { defineStore } from "pinia";
import { ref } from "vue";

export const useUiStore = defineStore("ui", () => {
  const sidebarCollapsed = ref(false);
  const currentRoute = ref("home");

  function toggleSidebar() {
    sidebarCollapsed.value = !sidebarCollapsed.value;
  }

  function setCurrentRoute(route: string) {
    currentRoute.value = route;
  }

  return {
    sidebarCollapsed,
    currentRoute,
    toggleSidebar,
    setCurrentRoute,
  };
});
