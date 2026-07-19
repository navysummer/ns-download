import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface CustomCategory {
  id: string;
  name: string;
  builtinType?: string;
  isBuiltin: boolean;
  visible: boolean;
  icon: string;
  extensions: string[];
  matchMode: "extension" | "regex";
  regexPattern: string;
}

export interface Task {
  id: string;
  url: string;
  file_name: string;
  save_dir: string;
  status: number;
  downloaded_bytes: number;
  total_bytes: number;
  error_message: string;
  created_at: string;
  completed_at: string;
  segments: number;
}

export interface Settings {
  saveDir: string;
  maxConcurrent: number;
  speedLimit: number;
  defaultThreads: number;
  autoMaxConnections: number;
  connPolicyCount: number;
  userAgent: string;
  revealFileCmd: string;
  retryCount: number;
  retryDelay: number;
  proxyType: string;
  proxyHost: string;
  proxyPort: number;
  proxyUsername: string;
  proxyPassword: string;
  proxyNoList: string;
  autoStartup: boolean;
  closeToTray: boolean;
  startMinimized: boolean;
  notifyOnComplete: boolean;
  keepAwake: boolean;
  torrentAssociated: boolean;
  theme: string;
  accentColor: string;
  language: string;
  uiScale: number;
  silentDownload: boolean;
  useServerTime: boolean;
  defaultQueueId: string;
  rememberLastSaveDir: boolean;
  showSidebarStatus: boolean;
  showSidebarQueues: boolean;
  showSidebarCategory: boolean;
  showTitlebarPauseAll: boolean;
  showTitlebarResumeAll: boolean;
  showTitlebarSettings: boolean;
  showTitlebarTheme: boolean;
  localServerEnabled: boolean;
  localServerPort: number;
  localServerToken: string;
  localServerTakeoverEnabled: boolean;
  localServerJsonrpcEnabled: boolean;
  localServerApiEnabled: boolean;
  localServerMcpEnabled: boolean;
  ffmpegPath: string;
  ytdlpPath: string;
  autoCheckUpdate: boolean;
  updateChannel: string;
  btPortStart: number;
  btPortEnd: number;
  btTrackerList: string;
  btTrackerSubUrls: string;
  ed2kEnableKad: boolean;
  ed2kEnableUpnp: boolean;
  ed2kListenPort: number;
  ed2kServerList: string;
  ed2kServerSubUrls: string;
}

export const useDownloadStore = defineStore("download", () => {
  const tasks = ref<Task[]>([]);
  const activeFilter = ref("all");

  const settings = ref<Settings>({
    saveDir: "/Downloads",
    maxConcurrent: 5,
    speedLimit: 0,
    defaultThreads: 0,
    autoMaxConnections: 16,
    connPolicyCount: 0,
    userAgent: "",
    revealFileCmd: "",
    retryCount: 5,
    retryDelay: 5,
    proxyType: "none",
    proxyHost: "",
    proxyPort: 1080,
    proxyUsername: "",
    proxyPassword: "",
    proxyNoList: "",
    autoStartup: false,
    closeToTray: false,
    startMinimized: false,
    notifyOnComplete: true,
    keepAwake: true,
    torrentAssociated: false,
    theme: "dark",
    accentColor: "blue",
    language: "",
    uiScale: 100,
    silentDownload: false,
    useServerTime: false,
    defaultQueueId: "default",
    rememberLastSaveDir: true,
    showSidebarStatus: true,
    showSidebarQueues: true,
    showSidebarCategory: true,
    showTitlebarPauseAll: true,
    showTitlebarResumeAll: true,
    showTitlebarSettings: true,
    showTitlebarTheme: true,
    localServerEnabled: false,
    localServerPort: 16891,
    localServerToken: "",
    localServerTakeoverEnabled: true,
    localServerJsonrpcEnabled: true,
    localServerApiEnabled: true,
    localServerMcpEnabled: true,
    ffmpegPath: "",
    ytdlpPath: "",
    autoCheckUpdate: true,
    updateChannel: "stable",
    btPortStart: 6881,
    btPortEnd: 6889,
    btTrackerList: "",
    btTrackerSubUrls: "",
    ed2kEnableKad: true,
    ed2kEnableUpnp: true,
    ed2kListenPort: 0,
    ed2kServerList: "",
    ed2kServerSubUrls: "",
  });

  const activeCount = computed(() =>
    tasks.value.filter((t) => t.status === 1).length
  );

  const downloadSpeed = computed(() => {
    const active = tasks.value.filter((t) => t.status === 1);
    if (active.length === 0) return "0 B/s";
    return `${active.length} 个下载中`;
  });

  const uploadSpeed = computed(() => "0 B/s");

  const filterTabs = computed(() => [
    { id: "all", label: "全部", count: tasks.value.length },
    { id: "active", label: "活跃", count: tasks.value.filter((t) => [0, 1, 5].includes(t.status)).length },
    { id: "completed", label: "已完成", count: tasks.value.filter((t) => t.status === 3).length },
    { id: "paused", label: "已暂停", count: tasks.value.filter((t) => t.status === 2).length },
    { id: "error", label: "错误", count: tasks.value.filter((t) => t.status === 4).length },
  ]);

  const filteredTasks = computed(() => {
    if (activeFilter.value === "all") return tasks.value;
    switch (activeFilter.value) {
      case "active":
        return tasks.value.filter((t) => [0, 1, 5].includes(t.status));
      case "completed":
        return tasks.value.filter((t) => t.status === 3);
      case "paused":
        return tasks.value.filter((t) => t.status === 2);
      case "error":
        return tasks.value.filter((t) => t.status === 4);
      default:
        return tasks.value;
    }
  });

  async function loadTasks() {
    try {
      tasks.value = await invoke<Task[]>("get_tasks");
    } catch (e) {
      console.error("Failed to load tasks:", e);
    }
  }

  async function addTask(spec: { url: string; save_dir: string; file_name?: string; segments?: number }) {
    try {
      await invoke("create_task", { spec });
      await loadTasks();
    } catch (e) {
      console.error("Failed to create task:", e);
    }
  }

  async function pauseTask(id: string) {
    try {
      await invoke("pause_task", { taskId: id });
      await loadTasks();
    } catch (e) {
      console.error("Failed to pause task:", e);
    }
  }

  async function resumeTask(id: string) {
    try {
      await invoke("resume_task", { taskId: id });
      await loadTasks();
    } catch (e) {
      console.error("Failed to resume task:", e);
    }
  }

  async function removeTask(id: string) {
    try {
      await invoke("remove_task", { taskId: id, deleteFiles: false });
      await loadTasks();
    } catch (e) {
      console.error("Failed to remove task:", e);
    }
  }

  async function pauseAll() {
    for (const t of tasks.value) {
      if (t.status === 1) await pauseTask(t.id);
    }
  }

  async function resumeAll() {
    for (const t of tasks.value) {
      if (t.status === 2) await resumeTask(t.id);
    }
  }

  const defaultCategories: CustomCategory[] = [
    { id: "all", name: "全部文件", builtinType: "all", isBuiltin: true, visible: true, icon: "FolderOpen", extensions: [], matchMode: "extension", regexPattern: "" },
    { id: "video", name: "视频", builtinType: "video", isBuiltin: true, visible: true, icon: "Video", extensions: ["mp4","mkv","avi","mov","wmv","flv","webm","m4v"], matchMode: "extension", regexPattern: "" },
    { id: "audio", name: "音频", builtinType: "audio", isBuiltin: true, visible: true, icon: "Music", extensions: ["mp3","flac","wav","aac","ogg","wma","m4a","opus"], matchMode: "extension", regexPattern: "" },
    { id: "document", name: "文档", builtinType: "document", isBuiltin: true, visible: true, icon: "FileText", extensions: ["pdf","doc","docx","xls","xlsx","ppt","pptx","txt","md","epub"], matchMode: "extension", regexPattern: "" },
    { id: "image", name: "图片", builtinType: "image", isBuiltin: true, visible: true, icon: "Image", extensions: ["jpg","jpeg","png","gif","bmp","svg","webp","ico","heic"], matchMode: "extension", regexPattern: "" },
    { id: "program", name: "程序", builtinType: "program", isBuiltin: true, visible: true, icon: "Package", extensions: ["exe","msi","dmg","app","deb","rpm","AppImage","pkg"], matchMode: "extension", regexPattern: "" },
    { id: "archive", name: "压缩包", builtinType: "archive", isBuiltin: true, visible: true, icon: "Archive", extensions: ["zip","rar","7z","tar","gz","bz2","xz","zst","iso"], matchMode: "extension", regexPattern: "" },
    { id: "other", name: "其他", builtinType: "other", isBuiltin: true, visible: true, icon: "File", extensions: [], matchMode: "extension", regexPattern: "" },
  ];

  const categories = ref<CustomCategory[]>(JSON.parse(JSON.stringify(defaultCategories)));

  function resetCategories() {
    categories.value = JSON.parse(JSON.stringify(defaultCategories));
  }

  function toggleCategoryVisibility(id: string) {
    const cat = categories.value.find(c => c.id === id);
    if (cat) cat.visible = !cat.visible;
  }

  function addCustomCategory(name: string, extensions: string[]) {
    const id = `custom_${Date.now()}`;
    categories.value.push({
      id, name, isBuiltin: false, visible: true, icon: "File",
      extensions, matchMode: "extension", regexPattern: "",
    });
  }

  function removeCategory(id: string) {
    const idx = categories.value.findIndex(c => c.id === id);
    if (idx >= 0 && !categories.value[idx].isBuiltin) {
      categories.value.splice(idx, 1);
    }
  }

  function moveCategory(from: number, to: number) {
    const arr = categories.value;
    if (to >= 0 && to < arr.length) {
      arr.splice(to, 0, arr.splice(from, 1)[0]);
    }
  }

  async function saveSettings() {
    try {
      await invoke("update_settings", { settings: settings.value });
    } catch (e) {
      console.error("Failed to save settings:", e);
    }
  }

  return {
    tasks,
    activeFilter,
    settings,
    activeCount,
    downloadSpeed,
    uploadSpeed,
    filterTabs,
    filteredTasks,
    loadTasks,
    addTask,
    pauseTask,
    resumeTask,
    removeTask,
    pauseAll,
    resumeAll,
    categories,
    resetCategories,
    toggleCategoryVisibility,
    addCustomCategory,
    removeCategory,
    moveCategory,
    saveSettings,
  };
});
