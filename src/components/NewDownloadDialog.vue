<script setup lang="ts">
import { ref, computed, onMounted, reactive } from "vue";
import { useDownloadStore } from "../lib/store";
import { X, Download, FolderOpen, Clock, FileDown, FileText, ChevronDown, Plus, CircleAlert } from "lucide-vue-next";
import { downloadDir } from "@tauri-apps/api/path";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import TorrentFilePickerDialog from "./TorrentFilePickerDialog.vue";

const props = defineProps<{ initialUrl?: string }>();
const emit = defineEmits<{ close: [] }>();
const store = useDownloadStore();

const url = ref(props.initialUrl || "");
const saveDir = ref("");
const rename = ref("");
const torrentFile = ref("");
const showTorrentPicker = ref(false);
const torrentMeta = ref<any>(null);

async function pickTorrent() {
  const selected = await open({
    multiple: false,
    filters: [{ name: 'Torrent', extensions: ['torrent'] }],
  });
  if (selected) {
    const path = selected as string;
    torrentFile.value = path;
    try {
      const { readFile } = await import("@tauri-apps/plugin-fs");
      const bytes = await readFile(path);
      const meta = await invoke("probe_torrent_file", { torrentBytes: Array.from(bytes) });
      torrentMeta.value = meta;
      showTorrentPicker.value = true;
    } catch (e) {
      console.error("Failed to probe torrent:", e);
    }
  }
}

function onTorrentFilesSelected(indices: number[]) {
  showTorrentPicker.value = false;
  // Set URL to the torrent file path, and prepare to create with torrent bytes
  url.value = torrentFile.value;
  selectedFileIndices.value = indices;
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

const selectedFileIndices = ref<number[]>([]);

async function pickTxt() {
  const selected = await open({
    multiple: false,
    filters: [{ name: 'URL List', extensions: ['txt'] }],
  });
  if (selected) {
    // Read the file and append URLs
    try {
      const { readTextFile } = await import("@tauri-apps/plugin-fs");
      const text = await readTextFile(selected as string);
      const existing = url.value.trim();
      url.value = existing ? existing + '\n' + text : text;
    } catch (e) {
      console.error("Failed to read file:", e);
    }
  }
}
const segments = ref("0");
const showAdvanced = ref(false);
const proxyUrl = ref("");
const userAgent = ref("");
const cookie = ref("");
const checksum = ref("");
const overwrite = ref(false);
const checksumAlgo = ref("sha-256");
interface HeaderRow { key: string; value: string }
const headerRows = reactive<HeaderRow[]>([]);

onMounted(async () => {
  try {
    saveDir.value = await downloadDir();
  } catch (e) {
    console.error("Failed to get download dir:", e);
    saveDir.value = store.settings.saveDir || "/Downloads";
  }
});

const urlCount = computed(() => {
  const t = url.value.trim();
  if (!t) return 0;
  return t.split('\n').filter(l => l.trim().length > 0 && !l.trim().startsWith('#')).length;
});

const segmentOptions = [
  { value: '0', label: 'Auto' },
  { value: '1', label: '1' },
  { value: '2', label: '2' },
  { value: '4', label: '4' },
  { value: '8', label: '8' },
  { value: '16', label: '16' },
  { value: '32', label: '32' },
  { value: '64', label: '64' },
];

const checksumAlgorithms = ['md5', 'sha-1', 'sha-256', 'sha-512'];

async function pickSaveDir() {
  const selected = await open({ directory: true, multiple: false, title: "选择保存目录" });
  if (selected) saveDir.value = selected;
}

const submitting = ref(false);

function submit(later = false) {
  if (submitting.value) return;
  if (!url.value.trim()) return;
  const entries = url.value.trim().split('\n').filter(l => l.trim() && !l.trim().startsWith('#'));
  if (entries.length === 0) return;
  const headers: Record<string, string> = {};
  for (const h of headerRows) {
    if (h.key.trim()) headers[h.key.trim()] = h.value;
  }
  // Guard against double-submit (rapid double-click fires submit twice).
  submitting.value = true;
  // Close the dialog immediately — do not wait for backend.
  emit("close");

  // Build spec and fire task creation asynchronously.
  (async () => {
    const hasTorrent = !!(torrentMeta.value && selectedFileIndices.value.length > 0);
    let torrentBytes: number[] | undefined;
    let torrentIndices: number[] | undefined;
    if (hasTorrent) {
      const { readFile } = await import("@tauri-apps/plugin-fs");
      torrentBytes = Array.from(await readFile(torrentFile.value));
      torrentIndices = selectedFileIndices.value;
    }
    for (const entry of entries) {
      const spec: any = {
        url: entry.trim(),
        save_dir: saveDir.value,
        file_name: rename.value || undefined,
        segments: parseInt(segments.value) || 0,
        start_paused: later,
      };
      if (later) {
        spec.queue_id = "later";
      }
      if (hasTorrent) {
        spec.torrent_file_bytes = torrentBytes;
        spec.selected_file_indices = torrentIndices;
      }
      if (proxyUrl.value.trim()) spec.proxy_url = proxyUrl.value.trim();
      if (userAgent.value.trim()) spec.user_agent = userAgent.value.trim();
      if (cookie.value.trim()) spec.cookies = cookie.value.trim();
      if (checksum.value.trim()) spec.checksum = checksumAlgo.value + '=' + checksum.value.trim();
      if (Object.keys(headers).length > 0) spec.extra_headers = headers;
      if (overwrite.value) spec.overwrite = true;
      await store.addTask(spec);
    }
  })();
}

function addHeader() {
  headerRows.push({ key: '', value: '' });
}

function removeHeader(index: number) {
  headerRows.splice(index, 1);
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center" :style="{ backgroundColor: 'rgba(0,0,0,0.5)' }" @click.self="emit('close')">
    <div class="flex w-full max-w-lg flex-col rounded-xl shadow-2xl" :style="{ backgroundColor: '#261C14', border: '1px solid #5A4330' }">
      <!-- Title -->
      <div class="flex items-center justify-between px-5 py-4" :style="{ borderBottom: '1px solid #5A4330' }">
        <div class="flex items-center gap-2.5">
          <div class="flex h-7 w-7 items-center justify-center rounded" :style="{ backgroundColor: 'rgba(var(--accent-rgb),0.18)' }">
            <Download class="h-3.5 w-3.5" :style="{ color: 'var(--accent)' }" />
          </div>
          <h2 class="text-sm font-semibold font-kai" :style="{ color: '#EDE0C8' }">新建下载</h2>
        </div>
        <button @click="emit('close')" class="rounded p-1 transition-colors" :style="{ color: '#9C8260' }">
          <X class="h-4 w-4" />
        </button>
      </div>

      <!-- Body -->
      <div class="space-y-4 overflow-y-auto px-5 py-4" style="max-height: 70vh;">
        <!-- URL textarea -->
        <div>
          <div class="mb-1.5 flex items-center justify-between">
            <span class="text-xs font-medium" :style="{ color: '#C9B393' }">下载地址 (URL)</span>
            <span v-if="urlCount > 0" class="text-2xs tabular-nums" :style="{ color: '#9C8260' }">{{ urlCount }} 个链接</span>
          </div>
          <textarea v-model="url" placeholder="https://example.com/file.zip"
            class="w-full resize-none rounded-md px-3 py-2 text-sm outline-none transition-colors"
            :style="{ backgroundColor: '#1A120E', border: '1px solid #5A4330', color: '#EDE0C8', minHeight: '100px' }" />
          <div class="mt-1.5 flex gap-2">
            <button @click="pickTorrent" class="flex items-center gap-1 rounded px-2 py-1 text-2xs transition-colors hover-bg" :style="{ color: 'var(--accent)' }">
              <FileDown class="h-3 w-3" /> 种子文件
            </button>
            <button @click="pickTxt" class="flex items-center gap-1 rounded px-2 py-1 text-2xs transition-colors hover-bg" :style="{ color: '#C9B393' }">
              <FileText class="h-3 w-3" /> 导入 TXT
            </button>
          </div>
        </div>

        <!-- Save dir -->
        <div>
          <span class="mb-1.5 block text-xs font-medium" :style="{ color: '#C9B393' }">保存到</span>
          <div class="flex gap-2">
            <input v-model="saveDir"
              class="flex-1 rounded-md px-3 py-2 text-sm outline-none transition-colors"
              :style="{ backgroundColor: '#1A120E', border: '1px solid #5A4330', color: '#EDE0C8' }" />
            <button @click="pickSaveDir" class="flex items-center justify-center rounded-md px-3 transition-colors hover-bg-surface2"
              :style="{ border: '1px solid #5A4330' }">
              <FolderOpen class="h-4 w-4" :style="{ color: '#C9B393' }" />
            </button>
          </div>
        </div>

        <!-- Threads + Rename row -->
        <div class="flex gap-3">
          <div class="flex-1">
            <span class="mb-1.5 block text-xs font-medium" :style="{ color: '#C9B393' }">线程数</span>
            <select v-model="segments"
              class="w-full rounded-md px-3 py-2 text-sm outline-none transition-colors"
              :style="{ backgroundColor: '#1A120E', border: '1px solid #5A4330', color: '#EDE0C8' }">
              <option v-for="opt in segmentOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
            </select>
          </div>
          <div class="flex-[2]">
            <span class="mb-1.5 block text-xs font-medium" :style="{ color: '#C9B393' }">重命名</span>
            <input v-model="rename" placeholder="可选"
              class="w-full rounded-md px-3 py-2 text-sm outline-none transition-colors"
              :style="{ backgroundColor: '#1A120E', border: '1px solid #5A4330', color: '#EDE0C8' }" />
          </div>
        </div>

        <!-- Overwrite checkbox -->
        <label class="flex cursor-pointer items-center gap-2 text-xs" :style="{ color: '#C9B393' }">
          <input type="checkbox" v-model="overwrite" class="h-3.5 w-3.5 rounded" :style="{ accentColor: 'var(--accent)' }" />
          如果文件已存在，覆盖原文件（跳过自动重命名）
        </label>

        <!-- Advanced toggle -->
        <button @click="showAdvanced = !showAdvanced" class="flex items-center gap-1 text-xs font-medium transition-colors" :style="{ color: '#9C8260' }">
          <ChevronDown class="h-3 w-3" :class="{ 'rotate-180': showAdvanced }" />
          高级选项
        </button>

        <!-- Advanced options -->
        <template v-if="showAdvanced">
          <!-- Proxy -->
          <div>
            <div class="mb-1 flex items-center gap-1.5">
              <span class="text-xs font-medium" :style="{ color: '#C9B393' }">代理地址</span>
              <CircleAlert class="h-3 w-3" :style="{ color: '#9C8260' }" />
            </div>
            <p class="mb-1.5 text-2xs" :style="{ color: '#9C8260' }">支持 http(s) 代理，格式为 http://地址:端口</p>
            <input v-model="proxyUrl" placeholder="http://127.0.0.1:1080"
              class="w-full rounded-md px-3 py-2 text-sm outline-none transition-colors"
              :style="{ backgroundColor: '#1A120E', border: '1px solid #5A4330', color: '#EDE0C8' }" />
          </div>

          <!-- User-Agent -->
          <div>
            <span class="mb-1 block text-xs font-medium" :style="{ color: '#C9B393' }">User-Agent</span>
            <p class="mb-1.5 text-2xs" :style="{ color: '#9C8260' }">自定义请求头，覆盖默认 User-Agent</p>
            <input v-model="userAgent" placeholder="Mozilla/5.0 ..."
              class="w-full rounded-md px-3 py-2 text-sm outline-none transition-colors"
              :style="{ backgroundColor: '#1A120E', border: '1px solid #5A4330', color: '#EDE0C8' }" />
          </div>

          <!-- Cookie -->
          <div>
            <span class="mb-1 block text-xs font-medium" :style="{ color: '#C9B393' }">Cookie</span>
            <p class="mb-1.5 text-2xs" :style="{ color: '#9C8260' }">设置请求 Cookie</p>
            <textarea v-model="cookie" placeholder="name=value; name2=value2"
              class="w-full resize-none rounded-md px-3 py-2 text-sm outline-none transition-colors"
              :style="{ backgroundColor: '#1A120E', border: '1px solid #5A4330', color: '#EDE0C8', minHeight: '48px' }" />
          </div>

          <!-- Checksum -->
          <div>
            <span class="mb-1 block text-xs font-medium" :style="{ color: '#C9B393' }">哈希校验</span>
            <p class="mb-1.5 text-2xs" :style="{ color: '#9C8260' }">下载完成后校验文件完整性</p>
            <div class="flex gap-2">
              <select v-model="checksumAlgo"
                class="w-28 rounded-md px-3 py-2 text-sm outline-none transition-colors"
                :style="{ backgroundColor: '#1A120E', border: '1px solid #5A4330', color: '#EDE0C8' }">
                <option v-for="algo in checksumAlgorithms" :key="algo" :value="algo">{{ algo }}</option>
              </select>
              <input v-model="checksum" placeholder="输入哈希值"
                class="flex-1 rounded-md px-3 py-2 text-sm outline-none transition-colors"
                :style="{ backgroundColor: '#1A120E', border: '1px solid #5A4330', color: '#EDE0C8' }" />
            </div>
          </div>

          <!-- Custom headers -->
          <div>
            <span class="mb-1 block text-xs font-medium" :style="{ color: '#C9B393' }">自定义请求头</span>
            <p class="mb-1.5 text-2xs" :style="{ color: '#9C8260' }">添加额外的 HTTP 请求头</p>
            <div v-for="(row, i) in headerRows" :key="i" class="mb-1.5 flex items-center gap-1.5">
              <input v-model="row.key" placeholder="Key"
                class="flex-[2] rounded-md px-2.5 py-1.5 text-sm outline-none transition-colors"
                :style="{ backgroundColor: '#1A120E', border: '1px solid #5A4330', color: '#EDE0C8' }" />
              <input v-model="row.value" placeholder="Value"
                class="flex-[3] rounded-md px-2.5 py-1.5 text-sm outline-none transition-colors"
                :style="{ backgroundColor: '#1A120E', border: '1px solid #5A4330', color: '#EDE0C8' }" />
              <button @click="removeHeader(i)" class="rounded p-1 transition-colors" :style="{ color: '#9C8260' }">
                <X class="h-3.5 w-3.5" />
              </button>
            </div>
            <button @click="addHeader" class="flex items-center gap-1 rounded px-2 py-1 text-2xs transition-colors hover-bg" :style="{ color: 'var(--accent)' }">
              <Plus class="h-3 w-3" /> 添加请求头
            </button>
          </div>
        </template>
      </div>

      <!-- Footer actions -->
      <div class="flex items-center justify-end gap-3 px-5 py-4" :style="{ borderTop: '1px solid #5A4330' }">
        <button @click="emit('close')"
          class="rounded-md px-4 py-1.5 text-sm font-medium transition-colors hover-bg-surface2" :style="{ color: '#C9B393' }">
          取消
        </button>
        <button @click="submit(true)" :disabled="!url.trim()"
          class="flex items-center gap-1.5 rounded-md px-4 py-1.5 text-sm font-medium transition-colors disabled:opacity-50"
          :style="{ backgroundColor: '#33271C', color: '#C9B393' }">
          <Clock class="h-3.5 w-3.5" /> 稍后下载
        </button>
        <button @click="submit(false)" :disabled="!url.trim()"
          class="flex items-center gap-1.5 rounded-md px-4 py-1.5 text-sm font-medium transition-colors disabled:opacity-50"
          :style="{ backgroundColor: 'var(--accent)', color: '#EDE0C8' }">
          <Download class="h-3.5 w-3.5" /> 开始下载
        </button>
      </div>
    </div>
  </div>

  <TorrentFilePickerDialog
    v-if="showTorrentPicker && torrentMeta"
    :meta="torrentMeta"
    :format-bytes="formatBytes"
    @close="showTorrentPicker = false"
    @confirm="onTorrentFilesSelected"
  />
</template>
