<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import { useRouter } from "vue-router";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { LogicalSize, LogicalPosition } from "@tauri-apps/api/dpi";
import Sidebar from "./components/Sidebar.vue";
import StatusBar from "./components/StatusBar.vue";
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
}

interface TasksSnapshotPayload {
  tasks: TaskItem[];
}

const store = useDownloadStore();
const router = useRouter();
const searchFocused = ref(false);

let unlistens: UnlistenFn[] = [];

onMounted(async () => {
  try {
    await invoke("init_engine");
    await store.loadSettings();

    unlistens.push(await listen<TasksSnapshotPayload>("tasks-snapshot", (event) => {
      store.tasks = event.payload.tasks;
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
          speed: p.speed,
          upload_speed: p.upload_speed_bps,
        };
      } else {
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

    // Keep awake while downloading
    watch(() => store.tasks.filter(t => t.status === 1).length, async (count) => {
      if (store.settings.keepAwake) {
        await store.preventSleep(count > 0);
      }
    });

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
    } catch (_) {}

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
      } catch (_) {}
    }, 2000);
    unlistens.push(() => clearInterval(saveInterval));

    // Close to tray
    unlistens.push(await mainWindow.onCloseRequested(async (event) => {
      if (store.settings.closeToTray) {
        event.preventDefault();
        await mainWindow.hide();
      }
    }));

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

  // Escape: blur search
  if (e.key === 'Escape') {
    if (isSearchFocused) {
      (document.activeElement as HTMLInputElement)?.blur();
    }
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
  <div class="flex h-screen flex-col" :style="{ backgroundColor: '#1C1C1E', color: '#F5F5F7' }">
    <div class="flex flex-1 overflow-hidden">
      <Sidebar />
      <div class="flex flex-1 flex-col overflow-hidden">
        <!-- Header Bar (40px) -->
        <header class="flex items-center gap-3 px-4" :style="{ height: '40px', borderBottom: '1px solid #48484A', backgroundColor: '#2C2C2E' }">
          <!-- Search bar -->
          <div class="relative flex-1" style="max-width: 320px;">
            <div class="flex items-center rounded-md px-2.5 transition-colors"
              :style="{
                backgroundColor: searchFocused ? 'rgba(59,130,246,0.08)' : '#1C1C1E',
                border: `1px solid ${searchFocused ? '#3B82F6' : '#48484A'}`,
                height: '30px',
              }"
            >
              <Search class="h-3.5 w-3.5 shrink-0" :style="{ color: searchFocused ? '#3B82F6' : '#8E8E93' }" />
              <input
                v-model="store.searchQuery"
                placeholder="搜索任务…"
                class="ml-2 flex-1 bg-transparent text-sm outline-none"
                :style="{ color: '#F5F5F7' }"
                @focus="searchFocused = true"
                @blur="searchFocused = false"
              />
              <span v-if="!searchFocused && !store.searchQuery"
                class="rounded px-1.5 py-0.5 text-2xs font-medium"
                :style="{ backgroundColor: '#3A3A3C', color: '#8E8E93', border: '1px solid #48484A' }"
              >Ctrl+F</span>
            </div>
          </div>

          <div class="flex-1" />

          <!-- Settings -->
          <button @click="router.push('/settings')"
            class="flex items-center justify-center rounded p-1.5 transition-colors hover-bg"
            :style="{ color: '#8E8E93', width: '36px', height: '36px' }"
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

    <!-- Drop overlay -->
    <div v-if="showDropOverlay"
      class="pointer-events-none fixed inset-0 z-[9999] flex items-center justify-center"
      :style="{ backgroundColor: 'rgba(59,130,246,0.15)' }"
    >
      <div class="rounded-2xl px-8 py-6 text-center"
        :style="{ backgroundColor: 'rgba(44,44,46,0.95)', border: '2px dashed #3B82F6' }"
      >
        <Download class="mx-auto h-8 w-8 mb-2" :style="{ color: '#3B82F6' }" />
        <div class="text-sm font-medium" :style="{ color: '#F5F5F7' }">释放链接以下载</div>
        <div class="text-xs mt-1" :style="{ color: '#8E8E93' }">支持 HTTP/HTTPS/FTP/Magnet 链接</div>
      </div>
    </div>
  </div>
</template>
