import { defineStore } from "pinia";
import { ref } from "vue";

export const useUiStore = defineStore("ui", () => {
  const currentRoute = ref("home");

  function setCurrentRoute(route: string) {
    currentRoute.value = route;
  }

  return {
    currentRoute,
    setCurrentRoute,
  };
});
