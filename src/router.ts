import { createRouter, createWebHistory } from "vue-router";
import TasksView from "./views/TasksView.vue";
import SettingsView from "./views/SettingsView.vue";

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: "/", redirect: "/tasks" },
    { path: "/tasks", name: "tasks", component: TasksView },
    { path: "/settings", name: "settings", component: SettingsView },
  ],
});

export default router;
