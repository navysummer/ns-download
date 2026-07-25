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
  speed: number;
  upload_speed: number;
  queue_id: string;
  priority?: boolean;
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
  btEnableDht: boolean;
  btEnableUpnp: boolean;
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
  const categoryFilter = ref("all");
  const searchQuery = ref("");
  const priorityTaskIds = ref<Set<string>>(new Set());
  const speedHistory = ref<number[]>([]);
  const sortField = ref<string>("");
  const sortOrder = ref<"asc" | "desc">("desc");
  const queueFilter = ref<string>("");

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
    btEnableDht: true,
    btEnableUpnp: true,
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

  const totalDownloadSpeed = computed(() =>
    tasks.value.filter((t) => t.status === 1).reduce((sum, t) => sum + (t.speed || 0), 0)
  );

  const totalUploadSpeed = computed(() =>
    tasks.value.filter((t) => t.status === 1).reduce((sum, t) => sum + (t.upload_speed || 0), 0)
  );

  const downloadSpeed = computed(() => {
    const speed = totalDownloadSpeed.value;
    if (speed <= 0) return "0 B/s";
    if (speed >= 1024 * 1024) return `${(speed / (1024 * 1024)).toFixed(1)} MB/s`;
    return `${Math.round(speed / 1024)} KB/s`;
  });

  const uploadSpeed = computed(() => {
    const speed = totalUploadSpeed.value;
    if (speed <= 0) return "0 B/s";
    if (speed >= 1024 * 1024) return `${(speed / (1024 * 1024)).toFixed(1)} MB/s`;
    return `${Math.round(speed / 1024)} KB/s`;
  });

  const filterTabs = computed(() => [
    { id: "all", label: "全部", count: tasks.value.length },
    { id: "active", label: "活跃", count: tasks.value.filter((t) => [0, 1, 5].includes(t.status)).length },
    { id: "completed", label: "已完成", count: tasks.value.filter((t) => t.status === 3).length },
    { id: "paused", label: "已暂停", count: tasks.value.filter((t) => t.status === 2).length },
    { id: "error", label: "错误", count: tasks.value.filter((t) => t.status === 4).length },
  ]);

  function getCategoryForTask(task: Task): string {
    const ext = task.file_name?.split('.').pop()?.toLowerCase() || '';
    for (const cat of categories.value) {
      if (cat.id === 'other') continue;
      if (cat.extensions.includes(ext)) return cat.id;
    }
    return 'other';
  }

  const filteredTasks = computed(() => {
    let result = tasks.value;
    if (searchQuery.value) {
      const q = searchQuery.value.toLowerCase();
      result = result.filter(t =>
        t.file_name?.toLowerCase().includes(q) ||
        t.url?.toLowerCase().includes(q) ||
        t.id?.toLowerCase().includes(q)
      );
    }
    if (queueFilter.value) {
      result = result.filter(t => t.queue_id === queueFilter.value || (!t.queue_id && queueFilter.value === 'default'));
    }
    if (categoryFilter.value && categoryFilter.value !== "all") {
      result = result.filter(t => getCategoryForTask(t) === categoryFilter.value);
    }
    if (activeFilter.value !== "all") {
      switch (activeFilter.value) {
        case "active":
          result = result.filter((t) => [0, 1, 5].includes(t.status)); break;
        case "completed":
          result = result.filter((t) => t.status === 3); break;
        case "paused":
          result = result.filter((t) => t.status === 2); break;
        case "error":
          result = result.filter((t) => t.status === 4); break;
      }
    }
    // Sort
    if (sortField.value) {
      const field = sortField.value as keyof Task;
      result = [...result].sort((a, b) => {
        let va: any = a[field], vb: any = b[field];
        if (typeof va === 'string') va = va.toLowerCase();
        if (typeof vb === 'string') vb = vb.toLowerCase();
        if (va == null) va = '';
        if (vb == null) vb = '';
        if (va < vb) return sortOrder.value === 'asc' ? -1 : 1;
        if (va > vb) return sortOrder.value === 'asc' ? 1 : -1;
        return 0;
      });
    }
    return result;
  });

  function recordSpeed(speed: number) {
    const h = speedHistory.value;
    h.push(speed);
    if (h.length > 60) h.shift();
    speedHistory.value = [...h];
  }

  function setSort(field: string) {
    if (sortField.value === field) {
      sortOrder.value = sortOrder.value === 'asc' ? 'desc' : 'asc';
    } else {
      sortField.value = field;
      sortOrder.value = field === 'downloaded_bytes' || field === 'speed' ? 'desc' : 'asc';
    }
  }

  function categoryCount(id: string): number {
    if (id === 'all') return tasks.value.length;
    return tasks.value.filter(t => getCategoryForTask(t) === id).length;
  }

  async function loadTasks() {
    try {
      tasks.value = await invoke<Task[]>("get_tasks");
    } catch (e) {
      console.error("Failed to load tasks:", e);
    }
  }

  async function addTask(spec: {
    url: string;
    save_dir: string;
    file_name?: string;
    segments?: number;
    torrent_file_bytes?: number[];
    selected_file_indices?: number[];
    proxy_url?: string;
    user_agent?: string;
    cookies?: string;
    referrer?: string;
    checksum?: string;
    extra_headers?: Record<string, string>;
  }) {
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

  async function setTaskPriority(taskId: string) {
    try {
      await invoke("set_task_priority", { taskId });
      const s = new Set(priorityTaskIds.value);
      if (s.has(taskId)) s.delete(taskId); else s.add(taskId);
      priorityTaskIds.value = s;
    } catch (e) {
      console.error("Failed to set task priority:", e);
    }
  }

  async function moveTaskToQueue(taskId: string, queueId: string) {
    try {
      await invoke("move_task_to_queue", { taskId, queueId });
      await loadTasks();
    } catch (e) {
      console.error("Failed to move task to queue:", e);
    }
  }

  function isPriorityTask(taskId: string): boolean {
    return priorityTaskIds.value.has(taskId);
  }

  const queueStates = ref<Record<string, boolean>>({ default: true, later: false });

  function toggleQueue(queueId: string) {
    const s = { ...queueStates.value };
    s[queueId] = !s[queueId];
    queueStates.value = s;
  }

  const queueTaskCounts = computed(() => {
    const counts: Record<string, number> = {};
    for (const t of tasks.value) {
      const qid = (t as any).queue_id || 'default';
      counts[qid] = (counts[qid] || 0) + 1;
    }
    for (const qid of Object.keys(queueStates.value)) {
      if (counts[qid] === undefined) counts[qid] = 0;
    }
    return counts;
  })

  async function revealInFolder(path: string) {
    try {
      await invoke("reveal_in_folder", { path });
    } catch (e) {
      console.error("Failed to reveal in folder:", e);
    }
  }

  async function sendNotification(title: string, body: string) {
    try {
      await invoke("send_notification", { title, body });
    } catch (e) {
      console.error("Failed to send notification:", e);
    }
  }

  async function preventSleep(prevent: boolean) {
    try {
      await invoke("prevent_sleep", { prevent });
    } catch (e) {
      console.error("Failed to toggle sleep:", e);
    }
  }

  async function shutdownSystem(action: string) {
    try {
      await invoke("shutdown_system", { action });
    } catch (e) {
      console.error("Failed to shutdown:", e);
    }
  }

  function setCategoryFilter(id: string) {
    categoryFilter.value = id;
  }

  function setActiveFilter(id: string) {
    activeFilter.value = id;
  }

  function setQueueFilter(id: string) {
    queueFilter.value = id;
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

  async function loadSettings() {
    try {
      const raw = await invoke<Record<string, string>>("load_settings");
      const s = settings.value;
      if (raw["default_save_dir"] !== undefined) s.saveDir = raw["default_save_dir"];
      if (raw["max_concurrent_tasks"] !== undefined) s.maxConcurrent = parseInt(raw["max_concurrent_tasks"]) || 5;
      if (raw["speed_limit_bytes"] !== undefined) s.speedLimit = parseInt(raw["speed_limit_bytes"]) || 0;
      if (raw["default_segments"] !== undefined) s.defaultThreads = parseInt(raw["default_segments"]) || 0;
      if (raw["auto_max_connections"] !== undefined) s.autoMaxConnections = parseInt(raw["auto_max_connections"]) || 16;
      if (raw["global_user_agent"] !== undefined) s.userAgent = raw["global_user_agent"];
      if (raw["max_auto_retries"] !== undefined) s.retryCount = parseInt(raw["max_auto_retries"]) ?? 3;
      if (raw["auto_retry_delay_secs"] !== undefined) s.retryDelay = parseInt(raw["auto_retry_delay_secs"]) || 5;
      if (raw["proxy_mode"] !== undefined) s.proxyType = raw["proxy_mode"];
      if (raw["proxy_host"] !== undefined) s.proxyHost = raw["proxy_host"];
      if (raw["proxy_port"] !== undefined) s.proxyPort = parseInt(raw["proxy_port"]) || 1080;
      if (raw["proxy_username"] !== undefined) s.proxyUsername = raw["proxy_username"];
      if (raw["proxy_password"] !== undefined) s.proxyPassword = raw["proxy_password"];
      if (raw["proxy_no_list"] !== undefined) s.proxyNoList = raw["proxy_no_list"];
      if (raw["close_to_tray"] !== undefined) s.closeToTray = raw["close_to_tray"] === "true";
      if (raw["auto_startup"] !== undefined) s.autoStartup = raw["auto_startup"] === "true";
      if (raw["start_minimized"] !== undefined) {
        s.startMinimized = raw["start_minimized"] === "true";
      } else {
        s.startMinimized = (raw["auto_startup"] === "true" && raw["close_to_tray"] === "true");
      }
      if (raw["notify_on_complete"] !== undefined) s.notifyOnComplete = raw["notify_on_complete"] !== "false";
      if (raw["use_server_time"] !== undefined) s.useServerTime = raw["use_server_time"] === "true";
      if (raw["bt_enable_dht"] !== undefined) s.btEnableDht = raw["bt_enable_dht"] === "true";
      if (raw["bt_enable_upnp"] !== undefined) s.btEnableUpnp = raw["bt_enable_upnp"] === "true";
      if (raw["bt_port_start"] !== undefined) s.btPortStart = parseInt(raw["bt_port_start"]) || 6881;
      if (raw["bt_port_end"] !== undefined) s.btPortEnd = parseInt(raw["bt_port_end"]) || 6891;
      if (raw["bt_custom_trackers"] !== undefined) s.btTrackerList = raw["bt_custom_trackers"];
      if (raw["bt_tracker_sub_urls"] !== undefined) s.btTrackerSubUrls = raw["bt_tracker_sub_urls"];
      if (raw["ed2k_enable_kad"] !== undefined) s.ed2kEnableKad = raw["ed2k_enable_kad"] === "true";
      if (raw["ed2k_enable_upnp"] !== undefined) s.ed2kEnableUpnp = raw["ed2k_enable_upnp"] === "true";
      if (raw["ed2k_listen_port"] !== undefined) s.ed2kListenPort = parseInt(raw["ed2k_listen_port"]) || 0;
      if (raw["ed2k_server_list"] !== undefined) s.ed2kServerList = raw["ed2k_server_list"];
      if (raw["ed2k_server_sub_urls"] !== undefined) s.ed2kServerSubUrls = raw["ed2k_server_sub_urls"];
      if (raw["local_server_enabled"] !== undefined) s.localServerEnabled = raw["local_server_enabled"] === "true";
      if (raw["local_server_port"] !== undefined) s.localServerPort = parseInt(raw["local_server_port"]) || 16891;
      if (raw["local_server_token"] !== undefined) s.localServerToken = raw["local_server_token"];
      if (raw["local_server_takeover_enabled"] !== undefined) s.localServerTakeoverEnabled = raw["local_server_takeover_enabled"] !== "false";
      if (raw["local_server_jsonrpc_enabled"] !== undefined) s.localServerJsonrpcEnabled = raw["local_server_jsonrpc_enabled"] !== "false";
      if (raw["local_server_api_enabled"] !== undefined) s.localServerApiEnabled = raw["local_server_api_enabled"] === "true";
      // UI-only settings (stored in DB as well)
      if (raw["ui_theme"] !== undefined) s.theme = raw["ui_theme"];
      if (raw["ui_accent_color"] !== undefined) s.accentColor = raw["ui_accent_color"];
      if (raw["ui_language"] !== undefined) s.language = raw["ui_language"];
      if (raw["ui_scale"] !== undefined) s.uiScale = parseInt(raw["ui_scale"]) || 100;
      if (raw["silent_download"] !== undefined) s.silentDownload = raw["silent_download"] === "true";
      if (raw["default_queue_id"] !== undefined) s.defaultQueueId = raw["default_queue_id"];
      if (raw["ffmpeg_path"] !== undefined) s.ffmpegPath = raw["ffmpeg_path"];
      if (raw["ytdlp_path"] !== undefined) s.ytdlpPath = raw["ytdlp_path"];
      if (raw["auto_check_update"] !== undefined) s.autoCheckUpdate = raw["auto_check_update"] !== "false";
      if (raw["update_channel"] !== undefined) s.updateChannel = raw["update_channel"];
      if (raw["reveal_file_cmd"] !== undefined) s.revealFileCmd = raw["reveal_file_cmd"];
      if (raw["torrent_associated"] !== undefined) s.torrentAssociated = raw["torrent_associated"] === "true";
      if (raw["keep_awake"] !== undefined) s.keepAwake = raw["keep_awake"] !== "false";
      if (raw["local_server_mcp_enabled"] !== undefined) s.localServerMcpEnabled = raw["local_server_mcp_enabled"] === "true";
      if (raw["conn_policy_count"] !== undefined) s.connPolicyCount = parseInt(raw["conn_policy_count"]) || 0;
      if (raw["remember_last_save_dir"] !== undefined) s.rememberLastSaveDir = raw["remember_last_save_dir"] === "true";
      if (raw["show_sidebar_status"] !== undefined) s.showSidebarStatus = raw["show_sidebar_status"] === "true";
      if (raw["show_sidebar_queues"] !== undefined) s.showSidebarQueues = raw["show_sidebar_queues"] === "true";
      if (raw["show_sidebar_category"] !== undefined) s.showSidebarCategory = raw["show_sidebar_category"] === "true";
    } catch (e) {
      console.error("Failed to load settings:", e);
    }
  }

  function settingsToMap(): Record<string, string> {
    const s = settings.value;
    return {
      "default_save_dir": s.saveDir,
      "max_concurrent_tasks": String(s.maxConcurrent),
      "speed_limit_bytes": String(s.speedLimit),
      "default_segments": String(s.defaultThreads),
      "auto_max_connections": String(s.autoMaxConnections),
      "global_user_agent": s.userAgent,
      "max_auto_retries": String(s.retryCount),
      "auto_retry_delay_secs": String(s.retryDelay),
      "proxy_mode": s.proxyType,
      "proxy_host": s.proxyHost,
      "proxy_port": String(s.proxyPort),
      "proxy_username": s.proxyUsername,
      "proxy_password": s.proxyPassword,
      "proxy_no_list": s.proxyNoList,
      "close_to_tray": s.closeToTray ? "true" : "false",
      "start_minimized": s.startMinimized ? "true" : "false",
      "auto_startup": s.autoStartup ? "true" : "false",
      "notify_on_complete": s.notifyOnComplete ? "true" : "false",
      "use_server_time": s.useServerTime ? "true" : "false",
      "bt_enable_dht": s.btEnableDht ? "true" : "false",
      "bt_enable_upnp": s.btEnableUpnp ? "true" : "false",
      "bt_port_start": String(s.btPortStart),
      "bt_port_end": String(s.btPortEnd),
      "bt_custom_trackers": s.btTrackerList,
      "bt_tracker_sub_urls": s.btTrackerSubUrls,
      "ed2k_enable_kad": s.ed2kEnableKad ? "true" : "false",
      "ed2k_enable_upnp": s.ed2kEnableUpnp ? "true" : "false",
      "ed2k_listen_port": String(s.ed2kListenPort),
      "ed2k_server_list": s.ed2kServerList,
      "ed2k_server_sub_urls": s.ed2kServerSubUrls,
      "local_server_enabled": s.localServerEnabled ? "true" : "false",
      "local_server_port": String(s.localServerPort),
      "local_server_token": s.localServerToken,
      "local_server_takeover_enabled": s.localServerTakeoverEnabled ? "true" : "false",
      "local_server_jsonrpc_enabled": s.localServerJsonrpcEnabled ? "true" : "false",
      "local_server_api_enabled": s.localServerApiEnabled ? "true" : "false",
      "ui_theme": s.theme,
      "ui_accent_color": s.accentColor,
      "ui_language": s.language,
      "ui_scale": String(s.uiScale),
      "silent_download": s.silentDownload ? "true" : "false",
      "default_queue_id": s.defaultQueueId,
      "ffmpeg_path": s.ffmpegPath,
      "ytdlp_path": s.ytdlpPath,
      "auto_check_update": s.autoCheckUpdate ? "true" : "false",
      "update_channel": s.updateChannel,
      "reveal_file_cmd": s.revealFileCmd,
      "torrent_associated": s.torrentAssociated ? "true" : "false",
      "keep_awake": s.keepAwake ? "true" : "false",
      "local_server_mcp_enabled": s.localServerMcpEnabled ? "true" : "false",
    };
  }

  let saveTimer: ReturnType<typeof setTimeout> | null = null;

  async function saveSettings() {
    try {
      await invoke("save_settings", { settings: settingsToMap() });
      // Sync auto-startup with system
      try {
        if (settings.value.autoStartup) {
          await invoke("plugin:autostart|enable");
        } else {
          await invoke("plugin:autostart|disable");
        }
      } catch (e2) {
        console.error("Failed to set auto-startup:", e2);
      }
    } catch (e) {
      console.error("Failed to save settings:", e);
    }
  }

  function saveSettingsDebounced() {
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => saveSettings(), 300);
  }

  return {
    tasks,
    activeFilter,
    categoryFilter,
    searchQuery,
    settings,
    activeCount,
    downloadSpeed,
    uploadSpeed,
    totalDownloadSpeed,
    totalUploadSpeed,
    filterTabs,
    filteredTasks,
    categoryCount,
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
    loadSettings,
    saveSettings,
    saveSettingsDebounced,
    settingsToMap,
    setTaskPriority,
    isPriorityTask,
    moveTaskToQueue,
    queueStates,
    queueTaskCounts,
    toggleQueue,
    revealInFolder,
    sendNotification,
    preventSleep,
    shutdownSystem,
    setCategoryFilter,
    setActiveFilter,
    queueFilter,
    setQueueFilter,
    sortField,
    sortOrder,
    setSort,
    speedHistory,
    recordSpeed,
  };
});
