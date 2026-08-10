<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import { useRouter } from "vue-router";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { LogicalSize, LogicalPosition } from "@tauri-apps/api/dpi";
import Sidebar from "./components/Sidebar.vue";
import StatusBar from "./components/StatusBar.vue";
import KeyboardShortcutsDialog from "./components/KeyboardShortcutsDialog.vue";
import { Search, Settings } from "lucide-vue-next";
import { useDownloadStore } from "./lib/store";

interface TaskPayload {
  task_id: string;
  status: number;
  downloaded_bytes: number;
  total_bytes: number;
  speed: number;
  file_name: string;
  save_dir: string;
  url: string;
  error_message: string;
  upload_speed_bps: number;
}

interface TaskItem {
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
  queue_id: string;
}

interface TasksSnapshotPayload {
  tasks: TaskItem[];
}

const store = useDownloadStore();
const router = useRouter();
const searchFocused = ref(false);
const showShortcuts = ref(false);

let unlistens: UnlistenFn[] = [];

onMounted(async () => {
  try {
    await invoke("init_engine");
    await store.loadSettings();
    await store.loadQueues();

    unlistens.push(await listen<TasksSnapshotPayload>("tasks-snapshot", (event) => {
      const seen = new Set<string>();
      store.tasks = event.payload.tasks.filter((t) => {
        if (!t.id || seen.has(t.id)) return false;
        seen.add(t.id);
        return true;
      });
    }));

    unlistens.push(await listen<TaskPayload>("task-progress", (event) => {
      const p = event.payload;
      const idx = store.tasks.findIndex(t => t.id === p.task_id);
      const prevStatus = idx >= 0 ? store.tasks[idx].status : -1;
      if (idx >= 0) {
        store.tasks[idx] = {
          ...store.tasks[idx],
          ...p,
          id: p.task_id,
          // 进度事件可能携带空 file_name（如 HLS 完成/错误更新），不能据此
          // 清空任务列表里已有的名称（与 task-meta-probed 的守卫一致）。
          file_name: p.file_name || store.tasks[idx].file_name,
          speed: p.speed,
          upload_speed: p.upload_speed_bps,
        };
      } else if (!store.tasks.some(t => t.id === p.task_id)) {
        store.tasks.push({
          id: p.task_id,
          url: p.url,
          file_name: p.file_name,
          save_dir: p.save_dir,
          status: p.status,
          downloaded_bytes: p.downloaded_bytes,
          total_bytes: p.total_bytes,
          error_message: p.error_message,
          created_at: "",
          completed_at: "",
          segments: 0,
          speed: p.speed,
          upload_speed: p.upload_speed_bps,
        });
      }
      // Record speed history for sparkline
      store.recordSpeed(store.totalDownloadSpeed);
      // Notification on complete
      if (p.status === 3 && prevStatus !== 3 && store.settings.notifyOnComplete) {
        store.sendNotification("下载完成", p.file_name || "任务已下载完成");
      }
    }));

    unlistens.push(await listen("segment-progress", (event: any) => {
      // Segment progress data available but not currently visualized in UI
      console.debug("Segment progress:", event.payload);
    }));

    unlistens.push(await listen("task-meta-probed", (event: any) => {
      const { task_id, file_name, total_bytes } = event.payload;
      const task = store.tasks.find(t => t.id === task_id);
      if (task) {
        task.file_name = file_name || task.file_name;
        if (total_bytes > 0) task.total_bytes = total_bytes;
      }
    }));

    unlistens.push(await listen("bt-data-finished", (event: any) => {
      console.debug("BT data finished:", event.payload.task_id);
    }));

    unlistens.push(await listen("queues-changed", (event: any) => {
      const payload = event.payload;
      if (Array.isArray(payload)) {
        const states: Record<string, boolean> = {};
        for (const q of payload) {
          states[q.queue_id] = q.is_running;
        }
        if (Object.keys(states).length > 0) {
          const labels = store.queueLabels;
          store.queueStates = states;
          store.queueLabels = labels;
        }
      }
    }));

    unlistens.push(await listen("task-queue-changed", (event: any) => {
      const { task_id, queue_id } = event.payload;
      const task = store.tasks.find(t => t.id === task_id);
      if (task) {
        task.queue_id = queue_id;
      }
    }));

    unlistens.push(await listen("priority-task-changed", (event: any) => {
      const { priority_task_id, auto_paused_count } = event.payload;
      if (priority_task_id) {
        const s = new Set(store.priorityTaskIds);
        s.add(priority_task_id);
        store.priorityTaskIds = s;
      }
    }));

    unlistens.push(await listen("queue-positions-changed", (event: any) => {
      const positions = event.payload;
      if (Array.isArray(positions)) {
        for (const p of positions) {
          const task = store.tasks.find(t => t.id === p.task_id);
          if (task) {
            task.segments = p.position;
          }
        }
      }
    }));

    unlistens.push(await listen("file-missing-changed", (event: any) => {
      const changes = event.payload;
      if (Array.isArray(changes)) {
        for (const { task_id, missing } of changes) {
          const task = store.tasks.find(t => t.id === task_id);
          if (task && missing) {
            console.warn(`Task ${task_id} files are missing`);
          }
        }
      }
    }));

    unlistens.push(await listen("plugin-hook-activity", (event: any) => {
      console.debug("Plugin hook activity:", event.payload);
    }));

    // Start API server if enabled
    if (store.settings.localServerEnabled) {
      try {
        await invoke("start_api_server");
      } catch (e) {
        console.error("Failed to start API server:", e);
      }
    }

    // Keep awake while downloading
    watch(() => store.tasks.filter(t => t.status === 1).length, async (count) => {
      if (store.settings.keepAwake) {
        await store.preventSleep(count > 0);
      }
    });

    // Apply theme
    watch(() => [store.settings.theme, store.settings.accentColor], () => applyTheme(), { immediate: true });
function applyTheme() {
      const root = document.documentElement;
      const isLight = store.settings.theme === 'classic-light' || store.settings.theme === 'light';
      root.style.setProperty('--app-bg', isLight ? '#f0e6d2' : '#0f0d0a');
      root.style.setProperty('--surface-bg', isLight ? '#faf6ef' : '#17130d');
      root.style.setProperty('--surface-bg-rgb', isLight ? '250,246,239' : '23,19,13');
      root.style.setProperty('--surface-bg2', isLight ? '#ede4d8' : '#211a11');
      root.style.setProperty('--surface-bg3', isLight ? '#e0d4c4' : '#2a2116');
      root.style.setProperty('--surface-dark', isLight ? '#e8dfd2' : 'rgba(38,29,18,0.6)');
      root.style.setProperty('--surface-border', isLight ? 'rgba(139,114,80,0.2)' : 'rgba(219,181,121,0.14)');
      root.style.setProperty('--text-primary', isLight ? '#2c1f14' : '#f0e6d2');
      root.style.setProperty('--text-secondary', isLight ? '#8c7358' : '#b8a98c');
      root.style.setProperty('--text-muted', isLight ? '#a08a6e' : '#8a7c63');
      root.style.setProperty('--text-muted-rgb', isLight ? '160,138,110' : '138,124,99');
      root.style.setProperty('--success', isLight ? '#4e7a5a' : '#6f9b8a');
      root.style.setProperty('--gold', isLight ? '#b8860b' : '#c9a25f');
      root.style.setProperty('--danger', isLight ? '#c0392b' : '#b0564a');
      root.style.setProperty('--accent', accentHex(store.settings.accentColor));
      root.style.setProperty('--accent-rgb', accentRgb(store.settings.accentColor));
      root.style.setProperty('--ui-scale', `${store.settings.uiScale / 100}`);
    }
    function accentHex(val: string): string {
      const m: Record<string, string> = {
        blue: '#3B82F6', green: '#22C55E', orange: '#F97316',
        purple: '#8B5CF6', pink: '#EC4899', red: '#D64531',
        teal: '#14B8A6', yellow: '#EAB308', cyan: '#06B6D4',
        '#3B82F6': '#3B82F6', '#22C55E': '#22C55E', '#F97316': '#F97316',
        '#06B6D4': '#06B6D4', '#8B5CF6': '#8B5CF6',
        '#EC4899': '#EC4899', '#D64531': '#D64531', '#EAB308': '#EAB308',
      };
      return m[val] || '#3B82F6';
    }
    function hexToRgb(hex: string): string {
      const v = parseInt(hex.replace('#', ''), 16);
      return `${(v >> 16) & 255}, ${(v >> 8) & 255}, ${v & 255}`;
    }
    function accentRgb(val: string): string {
      const h = accentHex(val);
      return hexToRgb(h);
    }

    // Window-level drag-and-drop
    document.addEventListener('dragover', onWindowDragOver);
    document.addEventListener('dragleave', onWindowDragLeave);
    document.addEventListener('drop', onWindowDrop);

    // Restore window state
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    const mainWindow = getCurrentWindow();
    try {
      const stored = localStorage.getItem('windowState');
      if (stored) {
        const ws = JSON.parse(stored);
        if (ws.width && ws.height) {
          await mainWindow.setSize(new LogicalSize(ws.width, ws.height));
        }
        if (typeof ws.x === 'number' && typeof ws.y === 'number') {
          await mainWindow.setPosition(new LogicalPosition(ws.x, ws.y));
        }
        if (ws.maximized) await mainWindow.maximize();
      }
    } catch (e) {
      console.warn("Failed to restore window state:", e);
    }

    // Save window state on resize/move
    const saveInterval = setInterval(async () => {
      try {
        const size = await mainWindow.innerSize();
        const pos = await mainWindow.outerPosition();
        const maximized = await mainWindow.isMaximized();
        localStorage.setItem('windowState', JSON.stringify({
          width: size.width, height: size.height,
          x: pos.x, y: pos.y,
          maximized,
        }));
      } catch (e) {
        console.warn("Failed to save window state:", e);
      }
    }, 2000);
    unlistens.push(() => clearInterval(saveInterval));

    // Dock badge for active downloading count
    watch(() => store.activeCount, async (count) => {
      try {
        await mainWindow.setBadgeCount(count);
      } catch (e) {
        console.warn("Failed to set badge count:", e);
      }
    }, { immediate: true });

    // Start minimized
    if (store.settings.startMinimized) {
      await mainWindow.hide();
    }
  } catch (e) {
    console.error("Failed to initialize engine:", e);
  }
});

// Keyboard shortcuts
function onKeyDown(e: KeyboardEvent) {
  const isSearchFocused = document.activeElement?.tagName === 'INPUT';
  const isTasksRoute = router.currentRoute.value.path === '/tasks';

  // Cmd+F / Ctrl+F: focus search
  if ((e.metaKey || e.ctrlKey) && e.key === 'f') {
    e.preventDefault();
    const input = document.querySelector<HTMLInputElement>('input[placeholder="搜索任务…"]');
    input?.focus();
    input?.select();
    return;
  }

  // Cmd+A / Ctrl+A: select all when in manage mode
  if ((e.metaKey || e.ctrlKey) && e.key === 'a' && isTasksRoute) {
    import('./views/TasksView.vue').then(m => {
      const el = document.querySelector<HTMLElement>('[data-manage-mode]');
      if (el) {
        el.click();
      }
    });
  }

  // Cmd+N / Ctrl+N: new download dialog
  if ((e.metaKey || e.ctrlKey) && e.key === 'n') {
    e.preventDefault();
    window.dispatchEvent(new CustomEvent('open-new-download'));
    return;
  }

  // Cmd+K / Ctrl+K: show keyboard shortcuts
  if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
    e.preventDefault();
    showShortcuts.value = true;
    return;
  }

  // Escape: blur search / close dialogs
  if (e.key === 'Escape') {
    if (isSearchFocused) {
      (document.activeElement as HTMLInputElement)?.blur();
    }
    if (showShortcuts.value) showShortcuts.value = false;
    return;
  }

  // Delete / Backspace: remove selected in manage mode
  if ((e.key === 'Delete' || e.key === 'Backspace') && !isSearchFocused && isTasksRoute) {
    // The TasksView manage mode handles this via event listeners
    window.dispatchEvent(new CustomEvent('batch-delete'));
  }

  // Space: toggle pause/resume for selected in manage mode
  if (e.key === ' ' && !isSearchFocused && isTasksRoute) {
    e.preventDefault();
    window.dispatchEvent(new CustomEvent('batch-toggle'));
  }
}

onUnmounted(() => {
  unlistens.forEach(fn => fn());
  document.removeEventListener('keydown', onKeyDown);
  document.removeEventListener('dragover', onWindowDragOver);
  document.removeEventListener('dragleave', onWindowDragLeave);
  document.removeEventListener('drop', onWindowDrop);
});

// Register keyboard shortcuts
document.addEventListener('keydown', onKeyDown);

// Window-level drag-and-drop for URLs
const showDropOverlay = ref(false);
let dropTimer: ReturnType<typeof setTimeout> | null = null;

function onWindowDragOver(e: DragEvent) {
  e.preventDefault();
  showDropOverlay.value = true;
  if (dropTimer) clearTimeout(dropTimer);
}

function onWindowDragLeave(e: DragEvent) {
  if (e.relatedTarget === null || (e.relatedTarget as Node)?.ownerDocument !== document) {
    dropTimer = setTimeout(() => { showDropOverlay.value = false; }, 200);
  }
}

function onWindowDrop(e: DragEvent) {
  e.preventDefault();
  showDropOverlay.value = false;
  const text = e.dataTransfer?.getData('text') || e.dataTransfer?.getData('text/plain');
  if (text && (text.startsWith('http://') || text.startsWith('https://') || text.startsWith('magnet:') || text.startsWith('ftp://'))) {
    router.push('/tasks');
    // The NewDownloadDialog will be shown via the drag-drop handler in TasksView
  }
}
</script>

<template>
  <div class="flex h-screen flex-col" :style="{ backgroundColor: 'var(--app-bg)', color: 'var(--text-primary)', zoom: `${store.settings.uiScale}%` }">
    <div class="flex flex-1 overflow-hidden">
      <Sidebar />
      <div class="flex flex-1 flex-col overflow-hidden">
        <!-- Header Bar (40px) -->
        <header class="flex items-center gap-3 px-4" :style="{ height: '40px', borderBottom: '1px solid var(--surface-border)', backgroundColor: 'var(--surface-bg)' }">
          <!-- Search bar -->
          <div class="relative flex-1" style="max-width: 320px;">
            <div class="flex items-center rounded-lg px-2.5 transition-colors"
              :style="{
                backgroundColor: searchFocused ? 'rgba(var(--accent-rgb),0.06)' : 'var(--app-bg)',
                border: `1px solid ${searchFocused ? 'var(--accent)' : 'var(--surface-border)'}`,
                height: '30px',
              }"
            >
              <Search class="h-3.5 w-3.5 shrink-0" :style="{ color: searchFocused ? 'var(--accent)' : 'var(--text-muted)' }" />
              <input
                v-model="store.searchQuery"
                placeholder="搜索任务…"
                class="ml-2 flex-1 bg-transparent text-sm outline-none"
                :style="{ color: 'var(--text-primary)' }"
                @focus="searchFocused = true"
                @blur="searchFocused = false"
              />
              <span v-if="!searchFocused && !store.searchQuery"
                class="rounded px-1.5 py-0.5 text-2xs font-medium"
                :style="{ backgroundColor: 'var(--surface-bg2)', color: 'var(--text-muted)', border: '1px solid var(--surface-border)' }"
              >Ctrl+F</span>
            </div>
          </div>

          <div class="flex-1" />

          <!-- Settings -->
          <button v-if="store.settings.showTitlebarSettings"
            @click="router.push('/settings')"
            class="flex items-center justify-center rounded-lg p-1.5 transition-colors hover-bg"
            :style="{ color: 'var(--text-muted)', width: '36px', height: '36px' }"
          >
            <Settings class="h-4 w-4" />
          </button>
        </header>

        <main class="flex-1 overflow-auto">
          <router-view />
        </main>
      </div>
    </div>
    <StatusBar />

    <KeyboardShortcutsDialog v-if="showShortcuts" @close="showShortcuts = false" />

    <!-- Drop overlay - 古风卷轴 -->
    <div v-if="showDropOverlay"
      class="pointer-events-none fixed inset-0 z-[9999] flex items-center justify-center"
      :style="{ backgroundColor: 'rgba(181,58,46,0.12)' }"
    >
      <div class="rounded-2xl px-8 py-6 text-center"
        :style="{ backgroundColor: 'rgba(var(--surface-bg-rgb),0.95)', border: '2px dashed var(--accent)' }"
      >
        <Download class="mx-auto h-8 w-8 mb-2" :style="{ color: 'var(--accent)' }" />
        <div class="text-sm font-medium" :style="{ color: 'var(--text-primary)' }">释放链接以下载</div>
        <div class="text-xs mt-1" :style="{ color: 'var(--text-muted)' }">支持 HTTP/HTTPS/FTP/Magnet 链接</div>
      </div>
    </div>
  </div>
</template>
