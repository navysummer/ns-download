<script setup lang="ts">
import { ref, onMounted, computed, watch, defineComponent, h } from "vue";
import { useDownloadStore } from "../lib/store";
import { downloadDir } from "@tauri-apps/api/path";
import { open, save } from "@tauri-apps/plugin-dialog";
import wxPayImg from "../assets/wx_pay.jpg";
import { invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";
import {
  Settings, Palette, Download, Globe, Server, Info, Puzzle,
  FolderOpen, PanelLeft, PanelTop, BellOff,
  Languages, SunMoon, Monitor, Sun, Moon, Maximize, RefreshCw, FileText, Shield,
  ChevronUp, ChevronDown, Eye, EyeOff, Trash2, Unplug, Key, List, Zap, Magnet, Network, Settings as SettingsIcon,
} from "lucide-vue-next";

const Divider = defineComponent({
  setup: () => () => h("div", { class: "h-px ml-4", style: "background-color: #3A3A3C;" }),
});

const SettingRow = defineComponent({
  props: { label: String, desc: String, modelValue: Boolean },
  emits: ["update:modelValue"],
  setup: (props, { emit }) => () => {
    const val = props.modelValue as boolean;
    return h("div", { class: "flex items-center justify-between px-4 py-2.5" }, [
      h("div", null, [
        h("label", { class: "text-sm font-medium", style: "color: #F5F5F7;" }, props.label),
        props.desc ? h("p", { class: "text-xs mt-0.5", style: "color: #8E8E93;" }, props.desc) : null,
      ]),
      h("button", {
        onClick: () => emit("update:modelValue", !val),
        class: "w-9 h-5 rounded-full transition-colors relative",
        style: { backgroundColor: val ? "var(--accent)" : "#48484A" },
      }, [
        h("span", {
          class: "absolute top-0.5 w-4 h-4 rounded-full bg-white transition-transform",
          style: { transform: val ? "translateX(16px)" : "translateX(2px)" },
        }),
      ]),
    ]);
  },
});

const ApiFeatureCard = defineComponent({
  props: { port: Number, path: String, label: String, desc: String, enabled: Boolean, modelValue: Boolean },
  emits: ["update:modelValue"],
  setup: (props, { emit }) => () => {
    const val = props.modelValue as boolean && props.enabled;
    return h("div", { class: "px-4 py-3" }, [
      h("div", { class: "flex items-center justify-between mb-2" }, [
        h("div", null, [
          h("label", { class: "text-sm font-medium", style: "color: #F5F5F7;" }, props.label),
          h("p", { class: "text-xs mt-0.5", style: "color: #8E8E93;" }, props.desc),
        ]),
        h("button", {
          onClick: () => emit("update:modelValue", !props.modelValue),
          class: "w-9 h-5 rounded-full transition-colors relative",
          style: { backgroundColor: val ? "var(--accent)" : "#48484A" },
        }, [
          h("span", {
            class: "absolute top-0.5 w-4 h-4 rounded-full bg-white transition-transform",
            style: { transform: val ? "translateX(16px)" : "translateX(2px)" },
          }),
        ]),
      ]),
      h("div", { class: "flex items-center gap-2" }, [
        h("span", { class: "text-xs shrink-0", style: "color: #A1A1A6;" }, "地址"),
        h("span", { class: "text-xs truncate flex-1 font-mono", style: "color: #8E8E93;" },
          `http://127.0.0.1:${props.port}${props.path}`),
      ]),
    ]);
  },
});

const store = useDownloadStore();
const activeCategory = ref("general");
const btSubTab = ref("basic");
const ed2kSubTab = ref("basic");

const categories = [
  { id: "general", label: "通用", icon: Settings },
  { id: "appearance", label: "外观", icon: Palette },
  { id: "download", label: "下载", icon: Download },
  { id: "proxy", label: "代理", icon: Globe },
  { id: "api", label: "API 服务", icon: Server },
  { id: "extensions", label: "扩展", icon: Puzzle },
  { id: "bitTorrent", label: "BitTorrent", icon: Magnet },
  { id: "ed2k", label: "ED2K", icon: Network },
  { id: "about", label: "关于", icon: Info },
];

const saveDir = ref(store.settings.saveDir);
const uaPreset = ref("default");

const uaPresets: Record<string, string> = {
  default: "",
  chrome: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
  firefox: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:120.0) Gecko/20100101 Firefox/120.0",
  edge: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Edg/120.0.0.0",
  safari: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.1 Safari/605.1.15",
};

const accentColors = [
  { id: "blue", color: "#3B82F6" },
  { id: "purple", color: "#8B5CF6" },
  { id: "pink", color: "#EC4899" },
  { id: "red", color: "#EF4444" },
  { id: "orange", color: "#F97316" },
  { id: "green", color: "#22C55E" },
  { id: "teal", color: "#14B8A6" },
  { id: "cyan", color: "#06B6D4" },
];

const themePresets = [
  { id: "classic-dark", label: "经典深色", color: "#3B82F6", appearance: "dark" },
  { id: "classic-light", label: "经典浅色", color: "#3B82F6", appearance: "light" },
  { id: "forest", label: "森林", color: "#22C55E", appearance: "dark" },
  { id: "sunset", label: "日落", color: "#F97316", appearance: "dark" },
  { id: "ocean", label: "海洋", color: "#06B6D4", appearance: "dark" },
  { id: "lavender", label: "薰衣草", color: "#8B5CF6", appearance: "dark" },
];

const serverPortDisplay = computed(() => store.settings.localServerPort);

watch(() => store.settings, () => {
  store.saveSettingsDebounced();
}, { deep: true });

watch(() => store.settings.localServerEnabled, async (enabled) => {
  try {
    if (enabled) {
      await invoke("start_api_server");
    } else {
      await invoke("stop_api_server");
    }
  } catch (e) {
    console.error("API server toggle failed:", e);
  }
});

function applyUaPreset() {
  const val = uaPresets[uaPreset.value];
  if (val !== undefined) store.settings.userAgent = val;
}

const showAddCategory = ref(false);
const newCategoryName = ref("");
const newCategoryExts = ref("");

// Extension detection
const ffmpegInfo = ref<{ status: string; version: string; path: string } | null>(null);
const ytdlpInfo = ref<{ status: string; version: string; path: string } | null>(null);

async function detectExtension(name: string) {
  try {
    const result = await invoke<string | null>("check_command_exists", { name });
    if (result) {
      if (name === "ffmpeg") {
        ffmpegInfo.value = { status: "系统", version: result, path: result };
      } else {
        ytdlpInfo.value = { status: "系统", version: result, path: result };
      }
    } else {
      if (name === "ffmpeg") {
        ffmpegInfo.value = { status: "未安装", version: "", path: "" };
      } else {
        ytdlpInfo.value = { status: "未安装", version: "", path: "" };
      }
    }
  } catch (e) {
    console.error(`Failed to detect ${name}:`, e);
    if (name === "ffmpeg") {
      ffmpegInfo.value = { status: "未安装", version: "", path: "" };
    } else {
      ytdlpInfo.value = { status: "未安装", version: "", path: "" };
    }
  }
}

// Update check
const updateChecking = ref(false);
const updateInfo = ref<{ has_update: boolean; latest_version: string; download_url: string; body: string; error_message: string } | null>(null);
async function checkUpdate() {
  updateChecking.value = true;
  updateInfo.value = null;
  try {
    updateInfo.value = await invoke("check_update", { currentVersion: await getVersion() });
  } catch (e) {
    updateInfo.value = { has_update: false, latest_version: "", download_url: "", body: "", error_message: String(e) };
  }
  updateChecking.value = false;
}

// Log export
async function exportLogs() {
  try {
    const dest = await save({ title: "导出日志", defaultPath: "ns-download-logs.zip" });
    if (dest) {
      await invoke("export_logs", { destPath: dest });
    }
  } catch (e) {
    console.error("Failed to export logs:", e);
  }
}

// Settings import/export
async function exportSettings() {
  try {
    const dest = await save({ title: "导出配置", defaultPath: "ns-download-settings.json" });
    if (dest) {
      const settings = store.settingsToMap();
      const json = JSON.stringify(settings, null, 2);
      const { writeTextFile } = await import("@tauri-apps/plugin-fs");
      await writeTextFile(dest, json);
    }
  } catch (e) {
    console.error("Failed to export settings:", e);
  }
}

async function importSettings() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'Settings JSON', extensions: ['json'] }],
    });
    if (selected) {
      const { readTextFile } = await import("@tauri-apps/plugin-fs");
      const text = await readTextFile(selected as string);
      const parsed = JSON.parse(text);
      // Apply to store and save
      for (const [key, value] of Object.entries(parsed)) {
        if (typeof value === 'string') {
          const setter = settingsKeyMap[key as keyof typeof settingsKeyMap];
          if (setter) setter(value);
        }
      }
      await store.saveSettings();
      // Reload settings to show imported values
      await store.loadSettings();
    }
  } catch (e) {
    console.error("Failed to import settings:", e);
  }
}

// Helper map for settings keys
const settingsKeyMap: Record<string, (v: string) => void> = {
  "default_save_dir": (v) => store.settings.saveDir = v,
  "max_concurrent_tasks": (v) => store.settings.maxConcurrent = parseInt(v) || 5,
  "speed_limit_bytes": (v) => store.settings.speedLimit = parseInt(v) || 0,
  "default_segments": (v) => store.settings.defaultThreads = parseInt(v) || 0,
  "auto_max_connections": (v) => store.settings.autoMaxConnections = parseInt(v) || 16,
  "global_user_agent": (v) => store.settings.userAgent = v,
  "max_auto_retries": (v) => store.settings.retryCount = parseInt(v) ?? 3,
  "auto_retry_delay_secs": (v) => store.settings.retryDelay = parseInt(v) || 5,
  "proxy_mode": (v) => store.settings.proxyType = v,
  "proxy_host": (v) => store.settings.proxyHost = v,
  "proxy_port": (v) => store.settings.proxyPort = parseInt(v) || 1080,
  "proxy_username": (v) => store.settings.proxyUsername = v,
  "proxy_password": (v) => store.settings.proxyPassword = v,
  "proxy_no_list": (v) => store.settings.proxyNoList = v,
  "close_to_tray": (v) => store.settings.closeToTray = v === "true",
  "start_minimized": (v) => store.settings.startMinimized = v === "true",
  "auto_startup": (v) => store.settings.autoStartup = v === "true",
  "notify_on_complete": (v) => store.settings.notifyOnComplete = v !== "false",
  "keep_awake": (v) => store.settings.keepAwake = v !== "false",
  "use_server_time": (v) => store.settings.useServerTime = v === "true",
  "proxy_type": (v) => store.settings.proxyProtocol = v,
  "bt_enable_dht": (v) => store.settings.btEnableDht = v === "true",
  "bt_enable_upnp": (v) => store.settings.btEnableUpnp = v === "true",
  "bt_port_start": (v) => store.settings.btPortStart = parseInt(v) || 6881,
  "bt_port_end": (v) => store.settings.btPortEnd = parseInt(v) || 6889,
  "bt_custom_trackers": (v) => store.settings.btTrackerList = v,
  "bt_tracker_sub_urls": (v) => store.settings.btTrackerSubUrls = v,
  "ed2k_enable_kad": (v) => store.settings.ed2kEnableKad = v === "true",
  "ed2k_enable_upnp": (v) => store.settings.ed2kEnableUpnp = v === "true",
  "ed2k_listen_port": (v) => store.settings.ed2kListenPort = parseInt(v) || 0,
  "ed2k_server_list": (v) => store.settings.ed2kServerList = v,
  "ed2k_server_sub_urls": (v) => store.settings.ed2kServerSubUrls = v,
  "ed2k_nodes_dat_url": (v) => store.settings.ed2kNodesDatUrl = v,
  "local_server_enabled": (v) => store.settings.localServerEnabled = v === "true",
  "local_server_port": (v) => store.settings.localServerPort = parseInt(v) || 17800,
  "local_server_token": (v) => store.settings.localServerToken = v,
  "local_server_takeover_enabled": (v) => store.settings.localServerTakeoverEnabled = v !== "false",
  "local_server_jsonrpc_enabled": (v) => store.settings.localServerJsonrpcEnabled = v !== "false",
  "local_server_api_enabled": (v) => store.settings.localServerApiEnabled = v === "true",
  "local_server_mcp_enabled": (v) => store.settings.localServerMcpEnabled = v === "true",
  "ui_theme": (v) => store.settings.theme = v,
  "ui_accent_color": (v) => store.settings.accentColor = v,
  "ui_language": (v) => store.settings.language = v,
  "ui_scale": (v) => store.settings.uiScale = parseInt(v) || 100,
  "silent_download": (v) => store.settings.silentDownload = v === "true",
  "default_queue_id": (v) => store.settings.defaultQueueId = v,
  "ffmpeg_path": (v) => store.settings.ffmpegPath = v,
  "ytdlp_path": (v) => store.settings.ytdlpPath = v,
  "auto_check_update": (v) => store.settings.autoCheckUpdate = v !== "false",
  "update_channel": (v) => store.settings.updateChannel = v,
  "reveal_file_cmd": (v) => store.settings.revealFileCmd = v,
  "torrent_associated": (v) => store.settings.torrentAssociated = v === "true",
  "conn_policy_count": (v) => store.settings.connPolicyCount = parseInt(v) || 0,
  "remember_last_save_dir": (v) => store.settings.rememberLastSaveDir = v === "true",
  "show_sidebar_status": (v) => store.settings.showSidebarStatus = v === "true",
  "show_sidebar_queues": (v) => store.settings.showSidebarQueues = v === "true",
  "show_sidebar_category": (v) => store.settings.showSidebarCategory = v === "true",
};

function confirmAddCategory() {
  const name = newCategoryName.value.trim();
  const exts = newCategoryExts.value.split(",").map(s => s.trim()).filter(Boolean);
  if (name) {
    store.addCustomCategory(name, exts);
    showAddCategory.value = false;
    newCategoryName.value = "";
    newCategoryExts.value = "";
  }
}

const threadOptions = [
  { value: "0", label: "自动" },
  { value: "1", label: "1" },
  { value: "2", label: "2" },
  { value: "4", label: "4" },
  { value: "8", label: "8" },
  { value: "16", label: "16" },
  { value: "32", label: "32" },
  { value: "64", label: "64" },
];

onMounted(async () => {
  if (!saveDir.value || saveDir.value === "/Downloads") {
    try {
      saveDir.value = await downloadDir();
      store.settings.saveDir = saveDir.value;
    } catch (e) {
      console.error("Failed to get download dir:", e);
    }
  }
  detectExtension("ffmpeg");
  detectExtension("yt-dlp");
});

async function pickDir() {
  const selected = await open({
    directory: true, multiple: false, title: "选择默认保存目录",
  });
  if (selected) {
    saveDir.value = selected;
    store.settings.saveDir = selected;
  }
}

const donateZoom = ref(0.5);
const donatePanX = ref(0);
const donatePanY = ref(0);
const donatePanning = ref(false);
const donatePanStartX = ref(0);
const donatePanStartY = ref(0);
const showDonate = ref(false);
const showChangelog = ref(false);

function donateZoomIn() { donateZoom.value = Math.min(donateZoom.value * 1.25, 10); }
function donateZoomOut() { donateZoom.value = Math.max(donateZoom.value / 1.25, 0.25); }
function donateZoomReset() { donateZoom.value = 1; donatePanX.value = 0; donatePanY.value = 0; }
function onDonateZoom(e: WheelEvent) {
  const delta = e.deltaY > 0 ? 0.9 : 1.1;
  donateZoom.value = Math.max(0.25, Math.min(10, donateZoom.value * delta));
}
function onDonatePanStart(e: MouseEvent) {
  donatePanning.value = true;
  donatePanStartX.value = e.clientX - donatePanX.value;
  donatePanStartY.value = e.clientY - donatePanY.value;
}
function onDonatePan(e: MouseEvent) {
  if (!donatePanning.value) return;
  donatePanX.value = e.clientX - donatePanStartX.value;
  donatePanY.value = e.clientY - donatePanStartY.value;
}
function onDonatePanEnd() { donatePanning.value = false; }

const proxyTesting = ref(false);
const proxyTestResult = ref<boolean | null>(null);
const proxyTestLatency = ref(0);
const proxyTestError = ref("");

async function testProxy() {
  const host = store.settings.proxyHost;
  const port = store.settings.proxyPort;
  if (!host || !port) return;
  proxyTesting.value = true;
  proxyTestResult.value = null;
  try {
    const result = await invoke<{ success: boolean; latency_ms: number; error_message: string }>("test_proxy", {
      proxyType: store.settings.proxyProtocol || 'http',
      proxyHost: host,
      proxyPort: port,
      proxyUsername: store.settings.proxyUsername,
      proxyPassword: store.settings.proxyPassword,
    });
    proxyTestResult.value = result.success;
    proxyTestLatency.value = result.latency_ms;
    proxyTestError.value = result.error_message;
  } catch (e) {
    proxyTestResult.value = false;
    proxyTestError.value = String(e);
  } finally {
    proxyTesting.value = false;
  }
}
</script>

<template>
  <div class="flex h-full">
    <!-- 侧边栏对齐 FluxDown _SettingsNavItem: 透明→hoverBg 背景 + 选中态 3px 右侧指示条 -->
    <div class="w-44 shrink-0 flex flex-col py-3 px-2 gap-0.5" style="background-color: #2C2C2E; border-right: 1px solid #48484A;">
      <div class="px-2 pb-3 mb-0.5 text-xs font-semibold uppercase tracking-wider" style="color: #8E8E93;">设置</div>
      <button
        v-for="cat in categories"
        :key="cat.id"
        @click="activeCategory = cat.id"
        class="group relative flex items-center gap-2.5 px-2.5 py-1.5 rounded-lg text-sm transition-colors text-left"
        :class="{ 'is-selected': activeCategory === cat.id }"
        :style="{
          backgroundColor: activeCategory === cat.id ? 'rgba(var(--accent-rgb),0.18)' : 'transparent',
          color: activeCategory === cat.id ? 'var(--accent)' : '#F5F5F7',
          fontWeight: activeCategory === cat.id ? 600 : 400,
        }"
        @mouseenter="($event.currentTarget as HTMLElement).style.backgroundColor = activeCategory === cat.id ? 'rgba(var(--accent-rgb),0.18)' : '#3A3A3C'"
        @mouseleave="($event.currentTarget as HTMLElement).style.backgroundColor = activeCategory === cat.id ? 'rgba(var(--accent-rgb),0.18)' : 'transparent'"
      >
        <component :is="cat.icon" class="w-4 h-4" :style="{ color: activeCategory === cat.id ? 'var(--accent)' : '#A1A1A6' }" />
        {{ cat.label }}
        <!-- 3px 右侧选中指示条 -->
        <span v-if="activeCategory === cat.id" class="absolute right-0 top-1/2 -translate-y-1/2 w-0.5 h-3.5 rounded-full" style="background-color: var(--accent);" />
      </button>
    </div>

    <div class="flex-1 overflow-y-auto">
      <div class="max-w-2xl p-6 space-y-6">

        <!-- ========== 通用 ========== -->
        <template v-if="activeCategory === 'general'">
          <h1 class="text-base font-semibold" style="color: #F5F5F7;">通用</h1>

          <div class="space-y-2">
            <h2 class="text-xs font-semibold tracking-wide pl-0.5" style="color: #A1A1A6;">启动与托盘</h2>
            <section class="rounded-xl overflow-hidden" style="background-color: #2C2C2E; border: 1px solid #48484A;">
              <SettingRow label="开机自启" desc="系统启动时自动运行" v-model="store.settings.autoStartup" />
              <Divider />
              <SettingRow label="关闭到托盘" desc="关闭窗口时最小化到系统托盘" v-model="store.settings.closeToTray" />
              <Divider />
              <SettingRow label="启动时最小化到托盘" desc="应用启动后自动最小化到托盘" v-model="store.settings.startMinimized" />
            </section>
          </div>

          <div class="space-y-2">
            <h2 class="text-xs font-semibold tracking-wide pl-0.5" style="color: #A1A1A6;">系统</h2>
            <section class="rounded-xl overflow-hidden" style="background-color: #2C2C2E; border: 1px solid #48484A;">
              <SettingRow label="Torrent 文件关联" desc="将 .torrent 文件与 ns-download 关联" v-model="store.settings.torrentAssociated" />
              <Divider />
              <SettingRow label="下载完成通知" desc="下载完成后显示系统通知" v-model="store.settings.notifyOnComplete" />
              <Divider />
              <SettingRow label="下载时保持唤醒" desc="下载进行时阻止系统休眠" v-model="store.settings.keepAwake" />
            </section>
          </div>

          <div class="space-y-2">
            <div class="pl-0.5 space-y-0.5">
              <h2 class="flex items-center gap-1.5 text-xs font-semibold tracking-wide" style="color: #A1A1A6;"><PanelLeft class="w-3 h-3" /> 侧边栏显示</h2>
              <p class="text-[11px]" style="color: #8E8E93;">控制侧边栏中各模块的可见性</p>
            </div>
            <section class="rounded-xl overflow-hidden" style="background-color: #2C2C2E; border: 1px solid #48484A;">
              <SettingRow label="状态标签" desc="显示下载中/已完成等状态标签" v-model="store.settings.showSidebarStatus" />
              <Divider />
              <SettingRow label="队列分组" desc="显示队列分组（默认/稍后下载）" v-model="store.settings.showSidebarQueues" />
              <Divider />
              <SettingRow label="分类列表" desc="显示文件分类列表" v-model="store.settings.showSidebarCategory" />
            </section>
          </div>

          <div class="space-y-2">
            <div class="pl-0.5 space-y-0.5">
              <h2 class="flex items-center gap-1.5 text-xs font-semibold tracking-wide" style="color: #A1A1A6;"><PanelTop class="w-3 h-3" /> 标题栏按钮</h2>
              <p class="text-[11px]" style="color: #8E8E93;">控制标题栏中各按钮的可见性</p>
            </div>
            <section class="rounded-xl overflow-hidden" style="background-color: #2C2C2E; border: 1px solid #48484A;">
              <SettingRow label="全部暂停" desc="在标题栏显示全部暂停按钮" v-model="store.settings.showTitlebarPauseAll" />
              <Divider />
              <SettingRow label="全部恢复" desc="在标题栏显示全部恢复按钮" v-model="store.settings.showTitlebarResumeAll" />
              <Divider />
              <SettingRow label="设置" desc="在标题栏显示设置按钮" v-model="store.settings.showTitlebarSettings" />
              <Divider />
              <SettingRow label="主题切换" desc="在标题栏显示主题切换按钮" v-model="store.settings.showTitlebarTheme" />
            </section>
          </div>

          <!-- 自定义分类：对齐 FluxDown _CustomCategoryManager -->
          <div class="space-y-2">
            <h2 class="flex items-center gap-1.5 text-xs font-semibold tracking-wide pl-0.5" style="color: #A1A1A6;"><FileText class="w-3 h-3" /> 自定义分类</h2>
            <section class="rounded-xl" style="background-color: #2C2C2E; border: 1px solid #48484A;">
              <div class="px-4 py-3 space-y-3">
                <div class="flex items-center justify-between">
                  <div>
                    <p class="text-xs" style="color: #8E8E93;">管理下载文件分类。拖拽或使用上下按钮调整优先级。</p>
                  </div>
                  <div class="flex gap-2">
                    <button @click="store.resetCategories()" class="h-7 px-2.5 rounded text-[11px] font-medium transition-colors" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #A1A1A6;">重置内置</button>
                    <button @click="showAddCategory = true" class="h-7 px-2.5 rounded text-[11px] font-medium transition-colors" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #A1A1A6;">+ 添加</button>
                  </div>
                </div>
                <div class="space-y-1">
                  <div v-for="(cat, i) in store.categories" :key="cat.id"
                    class="group flex items-center gap-2 px-3 py-2 rounded-lg transition-colors"
                    :style="{ backgroundColor: cat.visible ? 'transparent' : 'rgba(142,142,147,0.08)', border: '1px solid #3A3A3C' }">
                    <div class="flex flex-col gap-0.5">
                      <button @click="store.moveCategory(i, i - 1)" :disabled="i === 0" class="w-4 h-3 flex items-center justify-center disabled:opacity-30 hover:opacity-80" style="color: #8E8E93;"><ChevronUp class="w-3 h-3" /></button>
                      <button @click="store.moveCategory(i, i + 1)" :disabled="i === store.categories.length - 1" class="w-4 h-3 flex items-center justify-center disabled:opacity-30 hover:opacity-80" style="color: #8E8E93;"><ChevronDown class="w-3 h-3" /></button>
                    </div>
                    <div class="w-7 h-7 rounded-md flex items-center justify-center text-xs font-bold shrink-0" style="background-color: rgba(var(--accent-rgb),0.15); color: var(--accent);">
                      {{ cat.name.charAt(0) }}
                    </div>
                    <div class="flex-1 min-w-0">
                      <div class="flex items-center gap-1.5">
                        <span class="text-xs font-medium truncate" :style="{ color: cat.visible ? '#F5F5F7' : '#8E8E93' }">{{ cat.name }}</span>
                        <span v-if="cat.isBuiltin" class="text-[9px] px-1.5 py-0.5 rounded font-medium" style="background-color: rgba(var(--accent-rgb),0.12); color: var(--accent);">内置</span>
                        <EyeOff v-if="!cat.visible" class="w-3 h-3 shrink-0" style="color: #8E8E93;" />
                      </div>
                      <p v-if="cat.extensions.length > 0" class="text-[10px] truncate mt-0.5" style="color: #8E8E93;">.{{ cat.extensions.join(', .') }}</p>
                    </div>
                    <div class="hidden group-hover:flex items-center gap-1">
                      <button @click="store.toggleCategoryVisibility(cat.id)" class="w-6 h-6 rounded flex items-center justify-center hover:opacity-80" :style="{ color: cat.visible ? '#8E8E93' : 'var(--accent)' }">
                        <Eye v-if="cat.visible" class="w-3 h-3" />
                        <EyeOff v-else class="w-3 h-3" />
                      </button>
                      <button v-if="!cat.isBuiltin" @click="store.removeCategory(cat.id)" class="w-6 h-6 rounded flex items-center justify-center hover:opacity-80" style="color: #EF4444;">
                        <Trash2 class="w-3 h-3" />
                      </button>
                    </div>
                  </div>
                </div>
                <!-- 添加分类对话框 -->
                <div v-if="showAddCategory" class="p-4 rounded-lg space-y-3" style="background-color: #1C1C1E; border: 1px solid #48484A;">
                  <div>
                    <label class="text-xs font-medium mb-1 block" style="color: #F5F5F7;">分类名称</label>
                    <input v-model="newCategoryName" type="text" placeholder="例如：音乐" class="w-full h-8 rounded-md px-2.5 text-sm outline-none" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #F5F5F7;" />
                  </div>
                  <div>
                    <label class="text-xs font-medium mb-1 block" style="color: #F5F5F7;">文件扩展名（逗号分隔）</label>
                    <input v-model="newCategoryExts" type="text" placeholder="mp3, flac, wav" class="w-full h-8 rounded-md px-2.5 text-sm outline-none" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #F5F5F7;" />
                  </div>
                  <div class="flex justify-end gap-2 pt-1">
                    <button @click="showAddCategory = false; newCategoryName = ''; newCategoryExts = ''" class="h-7 px-3 rounded text-xs transition-colors" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #A1A1A6;">取消</button>
                    <button @click="confirmAddCategory" class="h-7 px-3 rounded text-xs transition-colors" style="background-color: var(--accent); color: #fff;">添加</button>
                  </div>
                </div>
              </div>
            </section>
          </div>
        </template>

        <!-- ========== 外观 ========== -->
        <template v-if="activeCategory === 'appearance'">
          <h1 class="text-base font-semibold" style="color: #F5F5F7;">外观</h1>

          <div class="space-y-2">
            <h2 class="flex items-center gap-1.5 text-xs font-semibold tracking-wide pl-0.5" style="color: #A1A1A6;"><Languages class="w-3 h-3" /> 语言</h2>
            <section class="rounded-xl overflow-hidden" style="background-color: #2C2C2E; border: 1px solid #48484A;">
              <div class="flex items-center justify-between px-4 py-2.5">
                <div><label class="text-sm font-medium" style="color: #F5F5F7;">界面语言</label><p class="text-xs mt-0.5" style="color: #8E8E93;">选择应用界面语言</p></div>
                <select v-model="store.settings.language" class="h-8 rounded-md px-2.5 text-sm outline-none transition-colors cursor-pointer" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #F5F5F7; min-width: 120px;">
                  <option value="system">跟随系统</option>
                  <option value="zh-CN">简体中文</option>
                </select>
              </div>
            </section>
          </div>

          <div class="space-y-2">
            <h2 class="flex items-center gap-1.5 text-xs font-semibold tracking-wide pl-0.5" style="color: #A1A1A6;"><SunMoon class="w-3 h-3" /> 主题</h2>
            <section class="rounded-xl overflow-hidden" style="background-color: #2C2C2E; border: 1px solid #48484A;">
              <!-- 主题模式：对齐 FluxDown _ThemeModeSelector（system/dark/light + 图标） -->
              <div class="px-4 py-3 space-y-3">
                <div><label class="text-sm font-medium" style="color: #F5F5F7;">主题模式</label><p class="text-xs mt-0.5" style="color: #8E8E93;">选择应用的色彩方案</p></div>
                <div class="flex gap-2">
                  <button v-for="opt in [{id:'system',icon:Monitor,label:'跟随系统'},{id:'light',icon:Sun,label:'浅色'},{id:'dark',icon:Moon,label:'深色'}]" :key="opt.id"
                    @click="store.settings.theme = opt.id"
                    class="flex items-center gap-1.5 px-3.5 py-2 rounded-lg text-xs font-medium transition-colors"
                    :style="{
                      backgroundColor: store.settings.theme === opt.id ? 'rgba(var(--accent-rgb),0.18)' : '#1C1C1E',
                      border: store.settings.theme === opt.id ? '1.5px solid rgba(var(--accent-rgb),0.4)' : '1px solid #48484A',
                      color: store.settings.theme === opt.id ? 'var(--accent)' : '#A1A1A6',
                    }"
                  ><component :is="opt.icon" class="w-3.5 h-3.5" /> {{ opt.label }}</button>
                </div>
              </div>
              <div class="h-px ml-4" style="background-color: #48484A;" />
              <!-- 主题选择：对齐 FluxDown _ThemeSelector（内置主题卡片） -->
              <div class="px-4 py-3 space-y-3">
                <div><label class="text-sm font-medium" style="color: #F5F5F7;">主题选择</label><p class="text-xs mt-0.5" style="color: #8E8E93;">选择预设主题方案</p></div>
                <div class="flex flex-wrap gap-2">
                  <button v-for="th in themePresets" :key="th.id" @click="store.settings.accentColor = th.color; store.settings.theme = th.appearance"
                    class="rounded-xl text-left transition-all hover:scale-[1.02]"
                    :style="{
                      width: '120px',
                      backgroundColor: '#1C1C1E',
                      border: (store.settings.accentColor === th.color ? '1.5px solid rgba(var(--accent-rgb),0.5)' : '1px solid #48484A'),
                      boxShadow: store.settings.accentColor === th.color ? '0 0 10px rgba(var(--accent-rgb),0.2)' : 'none',
                    }"
                  >
                    <div class="h-[52px] rounded-t-xl flex items-center justify-center" :style="{ backgroundColor: th.color }">
                      <span class="text-white text-lg font-bold" style="text-shadow: 0 1px 3px rgba(0,0,0,0.3);">{{ th.label.charAt(0) }}</span>
                    </div>
                    <div class="px-2 py-1.5">
                      <div class="text-xs font-medium truncate" :style="{ color: store.settings.accentColor === th.color ? 'var(--accent)' : '#F5F5F7' }">{{ th.label }}</div>
                      <div class="text-[10px] mt-0.5" style="color: #8E8E93;">{{ th.appearance === 'dark' ? '深色' : '浅色' }}</div>
                    </div>
                  </button>
                </div>
              </div>
              <div class="h-px ml-4" style="background-color: #48484A;" />
              <!-- 主题色选择：对齐 FluxDown _ColorSchemeSelector（圆点预设） -->
              <div class="px-4 py-3 space-y-3">
                <div><label class="text-sm font-medium" style="color: #F5F5F7;">主题色</label><p class="text-xs mt-0.5" style="color: #8E8E93;">选择强调色</p></div>
                <div class="flex gap-2.5">
                  <button v-for="ac in accentColors" :key="ac.id"
                    @click="store.settings.accentColor = ac.id"
                    :title="ac.id.charAt(0).toUpperCase() + ac.id.slice(1)"
                    class="w-7 h-7 rounded-full relative transition-transform hover:scale-110"
                    :style="{
                      backgroundColor: ac.color,
                      boxShadow: store.settings.accentColor === ac.id ? `0 0 8px ${ac.color}66` : 'none',
                    }"
                  >
                    <span v-if="store.settings.accentColor === ac.id" class="absolute inset-0 flex items-center justify-center">
                      <svg class="w-3 h-3 text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><polyline points="20 6 9 17 4 12"/></svg>
                    </span>
                    <span v-if="store.settings.accentColor === ac.id" class="absolute inset-0 rounded-full" style="border: 2.5px solid white;" />
                  </button>
                </div>
              </div>
            </section>
          </div>

          <div class="space-y-2">
            <h2 class="flex items-center gap-1.5 text-xs font-semibold tracking-wide pl-0.5" style="color: #A1A1A6;"><Maximize class="w-3 h-3" /> 界面缩放</h2>
            <section class="rounded-xl overflow-hidden" style="background-color: #2C2C2E; border: 1px solid #48484A;">
              <div class="flex items-center justify-between px-4 py-2.5">
                <div><label class="text-sm font-medium" style="color: #F5F5F7;">UI 缩放</label><p class="text-xs mt-0.5" style="color: #8E8E93;">调整界面元素大小</p></div>
                <div class="flex gap-1">
                  <button v-for="pct in [80,90,100,110,120,130,150]" :key="pct"
                    @click="store.settings.uiScale = pct"
                    class="px-2.5 py-1 rounded-md text-xs font-medium transition-colors"
                    :style="{
                      backgroundColor: store.settings.uiScale === pct ? 'rgba(var(--accent-rgb),0.18)' : 'transparent',
                      border: store.settings.uiScale === pct ? '1px solid rgba(var(--accent-rgb),0.4)' : '1px solid transparent',
                      color: store.settings.uiScale === pct ? 'var(--accent)' : '#A1A1A6',
                    }"
                  >{{ pct }}%</button>
                </div>
              </div>
            </section>
          </div>
        </template>

        <!-- ========== 下载 ========== -->
        <template v-if="activeCategory === 'download'">
          <h1 class="text-base font-semibold" style="color: #F5F5F7;">下载</h1>

          <div class="space-y-2">
            <h2 class="flex items-center gap-1.5 text-xs font-semibold tracking-wide pl-0.5" style="color: #A1A1A6;"><FolderOpen class="w-3 h-3" /> 保存位置</h2>
            <section class="rounded-xl overflow-hidden" style="background-color: #2C2C2E; border: 1px solid #48484A;">
              <div class="px-4 py-3 space-y-2">
                <label class="text-sm font-medium" style="color: #F5F5F7;">默认保存目录</label>
                <p class="text-xs" style="color: #8E8E93;">下载任务默认保存的位置</p>
                <div class="flex items-center h-9 rounded-lg cursor-pointer transition-colors" style="background-color: #1C1C1E; border: 1px solid #48484A;" @click="pickDir">
                  <span class="flex-1 px-3 text-sm truncate" :style="{ color: saveDir ? '#F5F5F7' : '#8E8E93' }">{{ saveDir || '选择默认保存目录' }}</span>
                  <div class="flex items-center gap-1.5 px-3 border-l h-5" style="border-color: #48484A;">
                    <FolderOpen class="w-3.5 h-3.5" style="color: #A1A1A6;" />
                    <span class="text-xs" style="color: #A1A1A6;">浏览</span>
                  </div>
                </div>
              </div>
              <Divider />
              <SettingRow label="记住上次保存目录" desc="新建下载时自动使用上一次的保存目录" v-model="store.settings.rememberLastSaveDir" />
            </section>
          </div>

          <div class="space-y-2">
            <h2 class="flex items-center gap-1.5 text-xs font-semibold tracking-wide pl-0.5" style="color: #A1A1A6;"><BellOff class="w-3 h-3" /> 行为</h2>
            <section class="rounded-xl overflow-hidden" style="background-color: #2C2C2E; border: 1px solid #48484A;">
              <SettingRow label="静默下载" desc="添加任务后直接开始下载，不弹出确认对话框" v-model="store.settings.silentDownload" />
              <Divider />
              <SettingRow label="使用服务器时间" desc="使用文件的服务器时间戳作为文件修改时间" v-model="store.settings.useServerTime" />
              <Divider />
              <div class="flex items-center justify-between px-4 py-2.5">
                <div><label class="text-sm font-medium" style="color: #F5F5F7;">默认队列</label><p class="text-xs mt-0.5" style="color: #8E8E93;">新建任务的默认队列</p></div>
                <select v-model="store.settings.defaultQueueId" class="h-8 rounded-md px-2.5 text-sm outline-none cursor-pointer" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #F5F5F7; min-width: 110px;">
                  <option value="default">默认</option>
                  <option value="later">稍后下载</option>
                </select>
              </div>
            </section>
          </div>

          <div class="space-y-2">
            <h2 class="flex items-center gap-1.5 text-xs font-semibold tracking-wide pl-0.5" style="color: #A1A1A6;"><Shield class="w-3 h-3" /> 连接</h2>
            <section class="rounded-xl overflow-hidden" style="background-color: #2C2C2E; border: 1px solid #48484A;">
              <div class="flex items-center justify-between px-4 py-2.5">
                <div><label class="text-sm font-medium" style="color: #F5F5F7;">默认线程数</label><p class="text-xs mt-0.5" style="color: #8E8E93;">每个下载任务的默认连接线程数</p></div>
                <select v-model.number="store.settings.defaultThreads" class="h-8 rounded-md px-2.5 text-sm outline-none transition-colors cursor-pointer" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #F5F5F7; min-width: 88px;">
                  <option v-for="opt in threadOptions" :key="opt.value" :value="Number(opt.value)">{{ opt.label }}</option>
                </select>
              </div>
              <Divider v-if="store.settings.defaultThreads === 0" />
              <div v-if="store.settings.defaultThreads === 0" class="flex items-center justify-between px-4 py-2.5">
                <div><label class="text-sm font-medium" style="color: #F5F5F7;">自动最大连接数</label><p class="text-xs mt-0.5" style="color: #8E8E93;">自动模式下的最大连接数上限</p></div>
                <div class="flex gap-1">
                  <button v-for="n in [4,8,16,32,64]" :key="n" @click="store.settings.autoMaxConnections = n"
                    class="px-2 py-1 rounded text-xs transition-colors"
                    :style="{ backgroundColor: store.settings.autoMaxConnections === n ? 'rgba(var(--accent-rgb),0.18)' : 'transparent', border: store.settings.autoMaxConnections === n ? '1px solid rgba(var(--accent-rgb),0.4)' : '1px solid transparent', color: store.settings.autoMaxConnections === n ? 'var(--accent)' : '#A1A1A6' }"
                  >{{ n }}</button>
                </div>
              </div>
              <Divider />
              <div class="flex items-center justify-between px-4 py-2.5">
                <div><label class="text-sm font-medium" style="color: #F5F5F7;">连接策略缓存</label><p class="text-xs mt-0.5" style="color: #8E8E93;">{{ store.settings.connPolicyCount > 0 ? `已缓存 ${store.settings.connPolicyCount} 条连接策略` : '暂无缓存数据' }}</p></div>
                <button @click="store.settings.connPolicyCount = 0" class="h-7 px-3 rounded text-xs transition-colors" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #A1A1A6;">清除</button>
              </div>
              <Divider />
              <div class="flex items-center justify-between px-4 py-2.5">
                <div><label class="text-sm font-medium" style="color: #F5F5F7;">最大并发任务数</label><p class="text-xs mt-0.5" style="color: #8E8E93;">同时下载的最大任务数</p></div>
                <div class="flex gap-1">
                  <button v-for="n in [1,2,3,5,8,10]" :key="n" @click="store.settings.maxConcurrent = n"
                    class="px-2 py-1 rounded text-xs transition-colors"
                    :style="{ backgroundColor: store.settings.maxConcurrent === n ? 'rgba(var(--accent-rgb),0.18)' : 'transparent', border: store.settings.maxConcurrent === n ? '1px solid rgba(var(--accent-rgb),0.4)' : '1px solid transparent', color: store.settings.maxConcurrent === n ? 'var(--accent)' : '#A1A1A6' }"
                  >{{ n }}</button>
                </div>
              </div>
              <Divider />
              <div class="px-4 py-3 space-y-2">
                <label class="text-sm font-medium" style="color: #F5F5F7;">速度限制</label>
                <p class="text-xs" style="color: #8E8E93;">0 表示不限速</p>
                <div class="flex items-center gap-2">
                  <input v-model.number="store.settings.speedLimit" type="number" min="0" class="flex-1 h-8 rounded-md px-2.5 text-sm outline-none transition-colors text-right" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #F5F5F7;" />
                  <span class="text-xs shrink-0" style="color: #8E8E93;">KB/s</span>
                </div>
              </div>
            </section>
          </div>

          <div class="space-y-2">
            <h2 class="text-xs font-semibold tracking-wide pl-0.5" style="color: #A1A1A6;">重试</h2>
            <section class="rounded-xl overflow-hidden" style="background-color: #2C2C2E; border: 1px solid #48484A;">
              <div class="flex items-center justify-between px-4 py-2.5">
                <div><label class="text-sm font-medium" style="color: #F5F5F7;">自动重试次数</label><p class="text-xs mt-0.5" style="color: #8E8E93;">下载失败时的最大重试次数</p></div>
                <select v-model.number="store.settings.retryCount" class="h-8 rounded-md px-2.5 text-sm outline-none transition-colors cursor-pointer" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #F5F5F7; min-width: 88px;">
                  <option :value="0">关闭</option>
                  <option :value="1">1 次</option>
                  <option :value="2">2 次</option>
                  <option :value="3">3 次</option>
                  <option :value="5">5 次</option>
                  <option :value="10">10 次</option>
                  <option :value="-1">不限</option>
                </select>
              </div>
              <Divider />
              <div class="px-4 py-3 space-y-2">
                <label class="text-sm font-medium" style="color: #F5F5F7;">重试间隔</label>
                <p class="text-xs" style="color: #8E8E93;">每次重试之间的等待时间</p>
                <div class="flex items-center gap-2">
                  <input v-model.number="store.settings.retryDelay" type="number" min="1" max="300" class="flex-1 h-8 rounded-md px-2.5 text-sm outline-none transition-colors text-right" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #F5F5F7;" />
                  <span class="text-xs" style="color: #8E8E93;">秒</span>
                </div>
              </div>
            </section>
          </div>

          <div class="space-y-2">
            <h2 class="text-xs font-semibold tracking-wide pl-0.5" style="color: #A1A1A6;">高级</h2>
            <section class="rounded-xl overflow-hidden" style="background-color: #2C2C2E; border: 1px solid #48484A;">
              <div class="px-4 py-3 space-y-2">
                <label class="text-sm font-medium" style="color: #F5F5F7;">User-Agent</label>
                <p class="text-xs" style="color: #8E8E93;">选择预设或自定义 User-Agent，留空使用默认值</p>
                <div class="flex gap-2">
                  <select v-model="uaPreset" class="h-9 rounded-md px-2.5 text-sm outline-none shrink-0" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #F5F5F7; width: 100px;" @change="applyUaPreset">
                    <option value="default">默认</option>
                    <option value="chrome">Chrome</option>
                    <option value="firefox">Firefox</option>
                    <option value="edge">Edge</option>
                    <option value="safari">Safari</option>
                    <option value="custom">自定义</option>
                  </select>
                  <input v-model="store.settings.userAgent" type="text" placeholder="Mozilla/5.0 ..." class="flex-1 h-9 rounded-md px-3 text-sm outline-none transition-colors" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #F5F5F7;" />
                </div>
              </div>
              <Divider />
              <div class="px-4 py-3 space-y-2">
                <label class="text-sm font-medium" style="color: #F5F5F7;">打开文件位置的命令</label>
                <p class="text-xs" style="color: #8E8E93;">点击"打开文件位置"时执行的命令，留空使用系统默认</p>
                <input v-model="store.settings.revealFileCmd" type="text" placeholder='open -R "{{path}}"' class="h-9 rounded-md px-3 text-sm outline-none transition-colors" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #F5F5F7;" />
              </div>
            </section>
          </div>
        </template>

        <!-- ========== 代理 ========== -->
        <template v-if="activeCategory === 'proxy'">
          <h1 class="text-base font-semibold" style="color: #F5F5F7;">代理</h1>
          <div class="space-y-2">
            <h2 class="text-xs font-semibold tracking-wide pl-0.5" style="color: #A1A1A6;">代理设置</h2>
            <section class="rounded-xl overflow-hidden" style="background-color: #2C2C2E; border: 1px solid #48484A;">
              <div class="px-4 py-3 space-y-4">
                <div class="flex gap-2">
                  <button v-for="opt in [{id:'none',icon:Unplug,label:'无代理'},{id:'system',icon:Monitor,label:'系统代理'},{id:'manual',icon:Settings,label:'手动代理'}]" :key="opt.id" @click="store.settings.proxyType = opt.id"
                    class="flex items-center justify-center gap-1.5 flex-1 h-9 rounded-lg text-xs font-medium transition-colors"
                    :style="{ backgroundColor: store.settings.proxyType === opt.id ? 'rgba(var(--accent-rgb),0.18)' : '#1C1C1E', border: store.settings.proxyType === opt.id ? '1px solid rgba(var(--accent-rgb),0.4)' : '1px solid #48484A', color: store.settings.proxyType === opt.id ? 'var(--accent)' : '#A1A1A6' }"
                  ><component :is="opt.icon" class="w-3.5 h-3.5" /> {{ opt.label }}</button>
                </div>
                <div v-if="store.settings.proxyType === 'system'" class="rounded-lg px-3 py-2.5" style="background-color: #1C1C1E; border: 1px solid #48484A;">
                  <p class="text-xs" style="color: #8E8E93;">使用系统代理设置</p>
                </div>
                <div v-if="store.settings.proxyType === 'manual'" class="space-y-4">
                  <div class="flex items-center gap-3">
                    <span class="text-xs w-16 shrink-0" style="color: #A1A1A6;">类型</span>
                    <select v-model="store.settings.proxyProtocol" @change="store.saveSettingsDebounced()" class="flex-1 h-8 rounded-md px-2.5 text-sm outline-none" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #F5F5F7;"><option value="http">HTTP</option><option value="https">HTTPS</option><option value="socks5">SOCKS5</option></select>
                  </div>
                  <div class="flex items-center gap-3">
                    <span class="text-xs w-16 shrink-0" style="color: #A1A1A6;">地址</span>
                    <input v-model="store.settings.proxyHost" placeholder="127.0.0.1" class="flex-1 h-8 rounded-md px-2.5 text-sm outline-none" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #F5F5F7;" />
                  </div>
                  <div class="flex items-center gap-3">
                    <span class="text-xs w-16 shrink-0" style="color: #A1A1A6;">端口</span>
                    <input v-model.number="store.settings.proxyPort" type="number" placeholder="1080" class="flex-1 h-8 rounded-md px-2.5 text-sm outline-none" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #F5F5F7;" />
                  </div>
                  <div class="flex items-center gap-3">
                    <span class="text-xs w-16 shrink-0" style="color: #A1A1A6;"><Key class="w-3 h-3" /></span>
                    <input v-model="store.settings.proxyUsername" placeholder="用户名（可选）" class="flex-1 h-8 rounded-md px-2.5 text-sm outline-none" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #F5F5F7;" />
                  </div>
                  <div class="flex items-center gap-3">
                    <span class="text-xs w-16 shrink-0" style="color: #A1A1A6;"><Key class="w-3 h-3" /></span>
                    <input v-model="store.settings.proxyPassword" type="password" placeholder="密码（可选）" class="flex-1 h-8 rounded-md px-2.5 text-sm outline-none" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #F5F5F7;" />
                  </div>
                  <div class="space-y-2">
                    <div class="flex items-center gap-3">
                      <span class="text-xs w-16 shrink-0" style="color: #A1A1A6;"><List class="w-3 h-3" /></span>
                      <span class="text-xs" style="color: #8E8E93;">绕过列表（每行一个，支持通配符）</span>
                    </div>
                    <textarea v-model="store.settings.proxyNoList" placeholder="localhost&#10;127.0.0.1&#10;*.local" rows="3" class="w-full rounded-md px-2.5 py-2 text-sm outline-none resize-none" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #F5F5F7;"></textarea>
                  </div>
                  <!-- 测试连接按钮（对齐 FluxDown _testProxy + _testResult） -->
                  <div class="flex items-center gap-3 pt-1">
                    <button @click="testProxy" :disabled="proxyTesting || !store.settings.proxyHost || !store.settings.proxyPort"
                      class="flex items-center gap-1.5 h-8 px-3 rounded-md text-xs font-medium transition-colors disabled:opacity-50"
                      :style="{ backgroundColor: '#1C1C1E', border: '1px solid #48484A', color: '#A1A1A6' }"
                    >
                      <Zap v-if="!proxyTesting" class="w-3.5 h-3.5" />
                      <svg v-else class="w-3.5 h-3.5 animate-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10" stroke-dasharray="31.4 31.4" stroke-linecap="round"/></svg>
                      {{ proxyTesting ? '测试中…' : '测试连接' }}
                    </button>
                    <span v-if="proxyTestResult === true" class="text-xs" style="color: #22C55E;">连接成功 ({{ proxyTestLatency }}ms)</span>
                    <span v-else-if="proxyTestResult === false" class="text-xs" style="color: #EF4444;">连接失败: {{ proxyTestError || '未知错误' }}</span>
                  </div>
                </div>
              </div>
            </section>
          </div>
        </template>

        <!-- ========== API 服务 ========== -->
        <template v-if="activeCategory === 'api'">
          <h1 class="text-base font-semibold" style="color: #F5F5F7;">API 服务</h1>
          <div class="space-y-2">
            <h2 class="text-xs font-semibold tracking-wide pl-0.5" style="color: #A1A1A6;">本地服务</h2>
            <section class="rounded-xl overflow-hidden" style="background-color: #2C2C2E; border: 1px solid #48484A;">
              <SettingRow label="启用本地 API 服务" desc="提供 HTTP API 供外部程序调用" v-model="store.settings.localServerEnabled" />
              <Divider />
              <div class="flex items-center justify-between px-4 py-2.5">
                <div><label class="text-sm font-medium" style="color: #F5F5F7;">监听端口</label><p class="text-xs mt-0.5" style="color: #8E8E93;">1024–65535</p></div>
                <input v-model.number="store.settings.localServerPort" type="number" min="1024" max="65535" class="h-8 w-24 rounded-md px-2.5 text-sm outline-none transition-colors text-center" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #F5F5F7;" :disabled="!store.settings.localServerEnabled" />
              </div>
              <Divider />
              <div class="px-4 py-3 space-y-2">
                <label class="text-sm font-medium" style="color: #F5F5F7;">访问令牌</label>
                <p class="text-xs" style="color: #8E8E93;">API 访问令牌，留空表示不验证</p>
                <input v-model="store.settings.localServerToken" type="text" placeholder="输入令牌或留空" class="h-9 rounded-md px-3 text-sm outline-none transition-colors" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #F5F5F7;" :disabled="!store.settings.localServerEnabled" />
              </div>
            </section>
          </div>

          <div class="space-y-2">
            <h2 class="text-xs font-semibold tracking-wide pl-0.5" style="color: #A1A1A6;">功能接口</h2>
            <section class="rounded-xl overflow-hidden" style="background-color: #2C2C2E; border: 1px solid #48484A;">
              <ApiFeatureCard :port="serverPortDisplay" path="/" label="下载接管" desc="浏览器下载接管" :enabled="store.settings.localServerEnabled" v-model="store.settings.localServerTakeoverEnabled" />
              <Divider />
              <ApiFeatureCard :port="serverPortDisplay" path="/jsonrpc" label="JSON-RPC" desc="JSON-RPC 接口" :enabled="store.settings.localServerEnabled" v-model="store.settings.localServerJsonrpcEnabled" />
              <Divider />
              <ApiFeatureCard :port="serverPortDisplay" path="/api/v1" label="REST API" desc="管理 API v1" :enabled="store.settings.localServerEnabled" v-model="store.settings.localServerApiEnabled" />
              <Divider />
              <ApiFeatureCard :port="serverPortDisplay" path="/mcp" label="MCP" desc="Model Context Protocol 接口" :enabled="store.settings.localServerEnabled" v-model="store.settings.localServerMcpEnabled" />
            </section>
          </div>
        </template>

        <!-- ========== 扩展 ========== -->
        <template v-if="activeCategory === 'extensions'">
          <h1 class="text-base font-semibold" style="color: #F5F5F7;">扩展</h1>

          <div class="space-y-2">
            <h2 class="flex items-center gap-1.5 text-xs font-semibold tracking-wide pl-0.5" style="color: #A1A1A6;"><Shield class="w-3 h-3" /> FFmpeg</h2>
            <section class="rounded-xl overflow-hidden" style="background-color: #2C2C2E; border: 1px solid #48484A;">
              <div class="px-4 py-3 space-y-3">
                <div class="flex items-center gap-2">
                  <span v-if="ffmpegInfo?.status === '系统'" class="inline-flex items-center px-2 py-0.5 rounded-full text-[10px] font-medium" style="background-color: rgba(34,197,94,0.15); color: #22C55E;">系统</span>
                  <span v-else class="inline-flex items-center px-2 py-0.5 rounded-full text-[10px] font-medium" style="background-color: rgba(var(--accent-rgb),0.15); color: var(--accent);">未安装</span>
                  <span v-if="ffmpegInfo?.version" class="text-xs" style="color: #8E8E93;">{{ ffmpegInfo.version }}</span>
                </div>
                <p v-if="ffmpegInfo?.path" class="text-xs" style="color: #8E8E93;">{{ ffmpegInfo.path }}</p>
                <p v-else class="text-xs" style="color: #8E8E93;">未在系统路径中找到 ffmpeg</p>
              </div>
              <Divider />
              <div class="px-4 py-3 space-y-2">
                <label class="text-sm font-medium" style="color: #F5F5F7;">手动路径</label>
                <p class="text-xs" style="color: #8E8E93;">指定 FFmpeg 可执行文件的路径</p>
                <div class="flex gap-2">
                  <input v-model="store.settings.ffmpegPath" type="text" placeholder="/usr/local/bin/ffmpeg" class="flex-1 h-9 rounded-md px-3 text-sm outline-none transition-colors" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #F5F5F7;" />
                  <button @click="store.saveSettingsDebounced()" class="h-9 px-3 rounded-md text-xs font-medium transition-colors" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #A1A1A6;">保存</button>
                </div>
              </div>
            </section>
          </div>

          <div class="space-y-2">
            <h2 class="flex items-center gap-1.5 text-xs font-semibold tracking-wide pl-0.5" style="color: #A1A1A6;"><Shield class="w-3 h-3" /> yt-dlp</h2>
            <section class="rounded-xl overflow-hidden" style="background-color: #2C2C2E; border: 1px solid #48484A;">
              <div class="px-4 py-3 space-y-3">
                <div class="flex items-center gap-2">
                  <span v-if="ytdlpInfo?.status === '系统'" class="inline-flex items-center px-2 py-0.5 rounded-full text-[10px] font-medium" style="background-color: rgba(34,197,94,0.15); color: #22C55E;">系统</span>
                  <span v-else class="inline-flex items-center px-2 py-0.5 rounded-full text-[10px] font-medium" style="background-color: rgba(var(--accent-rgb),0.15); color: var(--accent);">未安装</span>
                  <span v-if="ytdlpInfo?.version" class="text-xs" style="color: #8E8E93;">{{ ytdlpInfo.version }}</span>
                </div>
                <p v-if="ytdlpInfo?.path" class="text-xs" style="color: #8E8E93;">{{ ytdlpInfo.path }}</p>
                <p v-else class="text-xs" style="color: #8E8E93;">未在系统路径中找到 yt-dlp</p>
              </div>
              <Divider />
              <div class="px-4 py-3 space-y-2">
                <label class="text-sm font-medium" style="color: #F5F5F7;">手动路径</label>
                <p class="text-xs" style="color: #8E8E93;">指定 yt-dlp 可执行文件的路径</p>
                <div class="flex gap-2">
                  <input v-model="store.settings.ytdlpPath" type="text" placeholder="/usr/local/bin/yt-dlp" class="flex-1 h-9 rounded-md px-3 text-sm outline-none transition-colors" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #F5F5F7;" />
                  <button @click="store.saveSettingsDebounced()" class="h-9 px-3 rounded-md text-xs font-medium transition-colors" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #A1A1A6;">保存</button>
                </div>
              </div>
            </section>
          </div>
        </template>

        <!-- ========== BitTorrent ========== -->
        <template v-if="activeCategory === 'bitTorrent'">
          <h1 class="text-base font-semibold" style="color: #F5F5F7;">BitTorrent</h1>

          <!-- 子标签导航 -->
          <div class="flex gap-1 px-0.5 mb-4">
            <button v-for="tab in [{id:'basic',label:'基础'},{id:'tracker',label:'Tracker'}]" :key="tab.id"
              @click="btSubTab = tab.id"
              class="px-3 py-1.5 rounded-md text-xs font-medium transition-colors"
              :style="{
                backgroundColor: btSubTab === tab.id ? '#3A3A3C' : 'transparent',
                color: btSubTab === tab.id ? '#F5F5F7' : '#8E8E93',
              }"
            >{{ tab.label }}</button>
          </div>

          <!-- 基础 -->
          <template v-if="btSubTab === 'basic'">
            <section class="rounded-xl overflow-hidden" style="background-color: #2C2C2E; border: 1px solid #48484A;">
              <SettingRow label="启用 DHT 网络" desc="通过 DHT 网络发现更多用户" v-model="store.settings.btEnableDht" />
              <Divider />
              <SettingRow label="启用 UPnP" desc="自动进行 UPnP 端口映射" v-model="store.settings.btEnableUpnp" />
              <Divider />
              <div class="px-4 py-3 space-y-2">
                <label class="text-sm font-medium" style="color: #F5F5F7;">监听端口范围</label>
                <p class="text-xs" style="color: #8E8E93;">BT 协议使用的 TCP 端口范围</p>
                <div class="flex items-center gap-2">
                  <input v-model.number="store.settings.btPortStart" type="number" min="1024" max="65535" class="flex-1 h-9 rounded-md px-3 text-sm outline-none text-center" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #F5F5F7;" />
                  <span class="text-sm" style="color: #8E8E93;">–</span>
                  <input v-model.number="store.settings.btPortEnd" type="number" min="1024" max="65535" class="flex-1 h-9 rounded-md px-3 text-sm outline-none text-center" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #F5F5F7;" />
                </div>
              </div>
            </section>
            <div class="flex items-center gap-2 px-1 mt-2">
              <svg class="w-3 h-3 shrink-0" viewBox="0 0 24 24" fill="none" stroke="#8E8E93" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/></svg>
              <p class="text-[11px]" style="color: #8E8E93;">修改端口范围后需要重启应用才能生效</p>
            </div>
          </template>

          <!-- Tracker -->
          <template v-if="btSubTab === 'tracker'">
            <div class="space-y-3">
              <section class="rounded-xl overflow-hidden" style="background-color: #2C2C2E; border: 1px solid #48484A;">
                <div class="px-4 py-3 space-y-2">
                  <label class="text-sm font-medium" style="color: #F5F5F7;">Tracker 列表</label>
                  <p class="text-xs" style="color: #8E8E93;">每行一个 Tracker 地址</p>
                  <textarea v-model="store.settings.btTrackerList" rows="4" placeholder="udp://tracker.opentrackr.org:1337&#10;https://tracker.torrent.eu.org:443" class="w-full rounded-md px-2.5 py-2 text-sm outline-none resize-none" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #F5F5F7; font-family: monospace;"></textarea>
                </div>
              </section>
              <section class="rounded-xl overflow-hidden" style="background-color: #2C2C2E; border: 1px solid #48484A;">
                <div class="px-4 py-3 space-y-2">
                  <label class="text-sm font-medium" style="color: #F5F5F7;">Tracker 订阅</label>
                  <p class="text-xs" style="color: #8E8E93;">每行一个订阅 URL，自动更新 Tracker 列表</p>
                  <textarea v-model="store.settings.btTrackerSubUrls" rows="3" placeholder="https://raw.githubusercontent.com/ngosang/trackerslist/master/trackers_all.txt" class="w-full rounded-md px-2.5 py-2 text-sm outline-none resize-none" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #F5F5F7; font-family: monospace;"></textarea>
                </div>
              </section>
            </div>
          </template>
        </template>

        <!-- ========== ED2K ========== -->
        <template v-if="activeCategory === 'ed2k'">
          <h1 class="text-base font-semibold" style="color: #F5F5F7;">ED2K</h1>

          <!-- 子标签导航 -->
          <div class="flex gap-1 px-0.5 mb-4">
            <button v-for="tab in [{id:'basic',label:'基础'},{id:'servers',label:'服务器'}]" :key="tab.id"
              @click="ed2kSubTab = tab.id"
              class="px-3 py-1.5 rounded-md text-xs font-medium transition-colors"
              :style="{
                backgroundColor: ed2kSubTab === tab.id ? '#3A3A3C' : 'transparent',
                color: ed2kSubTab === tab.id ? '#F5F5F7' : '#8E8E93',
              }"
            >{{ tab.label }}</button>
          </div>

          <!-- 基础 -->
          <template v-if="ed2kSubTab === 'basic'">
            <section class="rounded-xl overflow-hidden" style="background-color: #2C2C2E; border: 1px solid #48484A;">
              <SettingRow label="启用 KAD 网络" desc="通过 KAD 网络发现资源" v-model="store.settings.ed2kEnableKad" />
              <Divider />
              <SettingRow label="启用 UPnP" desc="自动进行 UPnP 端口映射" v-model="store.settings.ed2kEnableUpnp" />
              <Divider />
              <div class="px-4 py-3 space-y-2">
                <label class="text-sm font-medium" style="color: #F5F5F7;">监听端口</label>
                <p class="text-xs" style="color: #8E8E93;">0 表示自动分配</p>
                <input v-model.number="store.settings.ed2kListenPort" type="number" min="0" max="65535" class="h-9 w-28 rounded-md px-3 text-sm outline-none text-center" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #F5F5F7;" />
              </div>
              <Divider />
              <div class="px-4 py-3 space-y-2">
                <label class="text-sm font-medium" style="color: #F5F5F7;">KAD nodes.dat 订阅</label>
                <p class="text-xs" style="color: #8E8E93;">KAD 网络节点列表下载地址</p>
                <input v-model="store.settings.ed2kNodesDatUrl" type="text" class="w-full h-9 rounded-md px-3 text-sm outline-none" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #F5F5F7;" />
              </div>
            </section>
          </template>

          <!-- 服务器 -->
          <template v-if="ed2kSubTab === 'servers'">
            <div class="space-y-3">
              <section class="rounded-xl overflow-hidden" style="background-color: #2C2C2E; border: 1px solid #48484A;">
                <div class="px-4 py-3 space-y-2">
                  <label class="text-sm font-medium" style="color: #F5F5F7;">服务器列表</label>
                  <p class="text-xs" style="color: #8E8E93;">每行一个 ED2K 服务器地址</p>
                  <textarea v-model="store.settings.ed2kServerList" rows="4" placeholder="ed2k://|server|192.168.1.1|4661|&#10;ed2k://|server|example.com|4661|" class="w-full rounded-md px-2.5 py-2 text-sm outline-none resize-none" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #F5F5F7; font-family: monospace;"></textarea>
                </div>
              </section>
              <section class="rounded-xl overflow-hidden" style="background-color: #2C2C2E; border: 1px solid #48484A;">
                <div class="px-4 py-3 space-y-2">
                  <label class="text-sm font-medium" style="color: #F5F5F7;">服务器订阅</label>
                  <p class="text-xs" style="color: #8E8E93;">每行一个订阅 URL，自动更新服务器列表</p>
                  <textarea v-model="store.settings.ed2kServerSubUrls" rows="3" placeholder="http://www.emule-security.net/serverlist.met&#10;http://upd.emule-security.org/server.met" class="w-full rounded-md px-2.5 py-2 text-sm outline-none resize-none" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #F5F5F7; font-family: monospace;"></textarea>
                </div>
              </section>
            </div>
          </template>
        </template>

        <!-- ========== 关于 ========== -->
        <template v-if="activeCategory === 'about'">
          <h1 class="text-base font-semibold" style="color: #F5F5F7;">关于</h1>

          <section class="rounded-xl" style="background-color: #2C2C2E; border: 1px solid #48484A;">
            <div class="p-5 space-y-4">
              <div class="flex items-center gap-4">
                <div class="w-12 h-12 rounded-xl flex items-center justify-center" style="background: linear-gradient(135deg, #0F172A 0%, #1E1B4B 100%); box-shadow: 0 2px 16px rgba(56,189,248,0.35); border: 1px solid #1E293B;">
              <svg width="28" height="28" viewBox="0 0 56 48" fill="none">
                <defs>
                  <linearGradient id="iconGrad" x1="0" y1="0" x2="1" y2="1">
                    <stop offset="0%" stop-color="#38BDF8" />
                    <stop offset="50%" stop-color="#818CF8" />
                    <stop offset="100%" stop-color="#C084FC" />
                  </linearGradient>
                  <linearGradient id="arrowGrad" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stop-color="#22D3EE" />
                    <stop offset="100%" stop-color="#38BDF8" />
                  </linearGradient>
                </defs>
                <g fill="none" stroke="url(#iconGrad)" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
                  <polyline points="4,36 4,12 17,36 17,12"/>
                  <path d="M24 19c0-3.5 2-5 4-5h3.5c3.5 0 5.5 2 5.5 5s-1.5 4-4 4h-3.5c-3.5 0-5.5 2-5.5 5s2 5 5.5 5h3.5c2 0 4-1.5 4-5"/>
                  <polyline points="40,36 40,12"/>
                  <path d="M40 12c7 0 12 6 12 12s-5 12-12 12"/>
                </g>
                <g fill="none" stroke="url(#arrowGrad)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <line x1="46" y1="22" x2="46" y2="31"/>
                  <polyline points="42,27 46,31 50,27"/>
                </g>
              </svg>
            </div>
                <div><div class="text-sm font-semibold" style="color: #F5F5F7;">ns-download</div><div class="text-xs mt-0.5" style="color: #8E8E93;">版本 0.1.0</div></div>
              </div>
              <div class="text-xs leading-relaxed" style="color: #8E8E93;">A modern download manager built with Tauri and Vue.</div>
              <div class="flex items-center gap-2 text-xs" style="color: #8E8E93;">
                <span>Tauri 2.x</span><span style="color: #48484A;">·</span><span>Vue 3.x</span><span style="color: #48484A;">·</span><span>Rust</span><span style="color: #48484A;">·</span><span>TypeScript</span>
              </div>
              <div class="flex gap-3 pt-1">
                <a href="https://github.com/navysummer/ns-download" target="_blank" class="text-xs underline underline-offset-2 hover:opacity-80" style="color: var(--accent);">GitHub</a>
                <a href="https://github.com/navysummer/ns-download/issues" target="_blank" class="text-xs underline underline-offset-2 hover:opacity-80" style="color: var(--accent);">反馈</a>
              </div>
            </div>
          </section>

          <div class="space-y-2">
            <h2 class="flex items-center gap-1.5 text-xs font-semibold tracking-wide pl-0.5" style="color: #A1A1A6;"><RefreshCw class="w-3 h-3" /> 软件更新</h2>
            <section class="rounded-xl overflow-hidden" style="background-color: #2C2C2E; border: 1px solid #48484A;">
              <SettingRow label="自动检查更新" desc="启动时自动检查新版本" v-model="store.settings.autoCheckUpdate" />
              <Divider />
              <div class="flex items-center justify-between px-4 py-2.5">
                <div><label class="text-sm font-medium" style="color: #F5F5F7;">更新频道</label><p class="text-xs mt-0.5" style="color: #8E8E93;">选择更新频道</p></div>
                <select v-model="store.settings.updateChannel" class="h-8 rounded-md px-2.5 text-sm outline-none" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #F5F5F7; min-width: 100px;"><option value="stable">稳定版</option><option value="frontier">前沿版</option></select>
              </div>
              <Divider />
              <div class="px-4 py-3">
                <button @click="checkUpdate" :disabled="updateChecking" class="h-8 px-4 rounded-md text-xs font-medium transition-colors hover:opacity-90" style="background-color: var(--accent); color: #fff;">
                  {{ updateChecking ? '检查中…' : '检查更新' }}
                </button>
                <div v-if="updateInfo" class="mt-2 text-xs" :style="{ color: updateInfo.has_update ? '#22C55E' : updateInfo.error_message ? '#EF4444' : '#8E8E93' }">
                  <template v-if="updateInfo.error_message">检查失败: {{ updateInfo.error_message }}</template>
                  <template v-else-if="updateInfo.has_update">
                    发现新版本 <a :href="updateInfo.download_url" target="_blank" style="color: var(--accent);">{{ updateInfo.latest_version }}</a>
                    <button v-if="updateInfo.body" @click="showChangelog = true" class="ml-2 underline underline-offset-2" style="color: var(--accent);">查看更新内容</button>
                  </template>
                  <template v-else>已是最新版本</template>
                </div>
              </div>
            </section>
          </div>

          <div class="space-y-2">
            <h2 class="flex items-center gap-1.5 text-xs font-semibold tracking-wide pl-0.5" style="color: #A1A1A6;"><FileText class="w-3 h-3" /> 日志导出</h2>
            <section class="rounded-xl" style="background-color: #2C2C2E; border: 1px solid #48484A;">
              <div class="px-4 py-3 space-y-3">
                <p class="text-xs" style="color: #8E8E93;">导出应用日志以排查问题</p>
                <button @click="exportLogs" class="h-8 px-4 rounded-md text-xs font-medium transition-colors" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #A1A1A6;">导出日志</button>
              </div>
            </section>
          </div>

          <div class="space-y-2">
            <h2 class="flex items-center gap-1.5 text-xs font-semibold tracking-wide pl-0.5" style="color: #A1A1A6;"><SettingsIcon class="w-3 h-3" /> 配置管理</h2>
            <section class="rounded-xl" style="background-color: #2C2C2E; border: 1px solid #48484A;">
              <div class="px-4 py-3 flex gap-2">
                <button @click="exportSettings" class="h-8 px-4 rounded-md text-xs font-medium transition-colors" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #A1A1A6;">导出配置</button>
                <button @click="importSettings" class="h-8 px-4 rounded-md text-xs font-medium transition-colors" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #A1A1A6;">导入配置</button>
              </div>
            </section>
          </div>

          <div class="space-y-2">
            <h2 class="flex items-center gap-1.5 text-xs font-semibold tracking-wide pl-0.5" style="color: #A1A1A6;"><svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/></svg> 支持项目</h2>
            <section class="rounded-xl overflow-hidden" style="background-color: #2C2C2E; border: 1px solid #48484A;">
              <div class="px-4 py-4 flex flex-col items-center gap-3">
                <p class="text-xs text-center leading-relaxed" style="color: #8E8E93;">如果您觉得 ns-download 对您有帮助，欢迎扫码支持开发者</p>
                <button @click="showDonate = true; donateZoom = 0.5; donatePanX = 0; donatePanY = 0" class="flex items-center gap-1.5 h-9 px-5 rounded-lg text-xs font-medium transition-colors hover:opacity-90" style="background-color: var(--accent); color: #fff;">
                  <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/></svg>
                  捐赠支持
                </button>
              </div>
            </section>
          </div>

          <!-- 捐赠弹窗 -->
          <Teleport to="body">
            <div v-if="showDonate" @click.self="showDonate = false" class="fixed inset-0 z-50 flex items-center justify-center" style="background-color: rgba(0,0,0,0.6);">
              <div class="relative rounded-2xl p-6" style="background-color: #2C2C2E; border: 1px solid #48484A;">
                <button @click="showDonate = false" class="absolute -top-3 -right-3 w-8 h-8 rounded-full flex items-center justify-center text-sm transition-colors hover:opacity-80" style="background-color: #3A3A3C; color: #F5F5F7; border: 1px solid #48484A;">✕</button>
                <div class="overflow-auto max-w-[90vw] max-h-[85vh] flex items-center justify-center" @wheel.prevent="onDonateZoom" @mousedown="onDonatePanStart" @mousemove="onDonatePan" @mouseup="onDonatePanEnd" @mouseleave="onDonatePanEnd" style="cursor: grab;">
                  <img :src="wxPayImg" alt="微信赞赏码" :style="{ transform: `scale(${donateZoom}) translate(${donatePanX}px, ${donatePanY}px)`, transition: donatePanning ? 'none' : 'transform 0.1s ease' }" class="rounded-lg select-none" draggable="false" />
                </div>
                <div class="flex items-center justify-center gap-3 mt-3">
                  <button @click="donateZoomIn" class="w-7 h-7 rounded flex items-center justify-center text-xs transition-colors" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #F5F5F7;">＋</button>
                  <span class="text-xs" style="color: #8E8E93;">{{ Math.round(donateZoom * 100) }}%</span>
                  <button @click="donateZoomOut" class="w-7 h-7 rounded flex items-center justify-center text-xs transition-colors" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #F5F5F7;" >−</button>
                  <button @click="donateZoomReset" class="w-7 h-7 rounded flex items-center justify-center text-xs transition-colors" style="background-color: #1C1C1E; border: 1px solid #48484A; color: #A1A1A6;">原图</button>
                </div>
              </div>
            </div>
          </Teleport>

          <!-- Changelog dialog -->
          <Teleport to="body">
            <div v-if="showChangelog && updateInfo" @click.self="showChangelog = false" class="fixed inset-0 z-50 flex items-center justify-center" style="background-color: rgba(0,0,0,0.6);">
              <div class="rounded-2xl overflow-hidden shadow-2xl" style="background-color: #2C2C2E; border: 1px solid #48484A; width: 480px; max-height: 70vh;">
                <div class="flex items-center justify-between px-4 py-3" style="border-bottom: 1px solid #48484A;">
                  <span class="text-sm font-semibold" style="color: #F5F5F7;">{{ updateInfo.latest_version }} 更新内容</span>
                  <button @click="showChangelog = false" class="w-7 h-7 rounded flex items-center justify-center text-sm transition-colors" style="color: #8E8E93;">✕</button>
                </div>
                <div class="overflow-y-auto p-4 text-xs leading-relaxed whitespace-pre-wrap" style="color: #A1A1A6; max-height: calc(70vh - 52px);">{{ updateInfo.body }}</div>
                <div class="flex justify-end px-4 py-3" style="border-top: 1px solid #48484A;">
                  <a :href="updateInfo.download_url" target="_blank"
                    class="rounded-md px-4 py-1.5 text-xs font-medium transition-colors"
                    style="background-color: var(--accent); color: #fff;">前往下载</a>
                </div>
              </div>
            </div>
          </Teleport>
        </template>

      </div>
    </div>
  </div>
</template>
