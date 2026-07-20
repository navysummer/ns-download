<script setup lang="ts">
import { ref, computed, onMounted, reactive } from "vue";
import { useDownloadStore } from "../lib/store";
import { X, Download, FolderOpen, Clock, FileDown, FileText, ChevronDown, Plus, CircleAlert } from "lucide-vue-next";
import { downloadDir } from "@tauri-apps/api/path";
import { open } from "@tauri-apps/plugin-dialog";

const emit = defineEmits<{ close: [] }>();
const store = useDownloadStore();

const url = ref("");
const saveDir = ref("");
const rename = ref("");
const segments = ref("0");
const showAdvanced = ref(false);
const proxyUrl = ref("");
const userAgent = ref("");
const cookie = ref("");
const checksum = ref("");
const checksumAlgo = ref("sha-256");
interface HeaderRow { key: string; value: string }
const headerRows = reactive<HeaderRow[]>([]);

onMounted(async () => {
  try {
    saveDir.value = await downloadDir();
  } catch {
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

async function submit(later = false) {
  if (!url.value.trim()) return;
  const entries = url.value.trim().split('\n').filter(l => l.trim() && !l.trim().startsWith('#'));
  for (const entry of entries) {
    await store.addTask({
      url: entry.trim(),
      save_dir: saveDir.value,
      file_name: rename.value || undefined,
      segments: parseInt(segments.value) || 0,
    });
  }
  emit("close");
}

function addHeader() {
  headerRows.push({ key: '', value: '' });
}

function removeHeader(index: number) {
  headerRows.splice(index, 1);
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center" :style="{ backgroundColor: 'rgba(0,0,0,0.4)' }" @click.self="emit('close')">
    <div class="flex w-full max-w-lg flex-col rounded-xl shadow-2xl" :style="{ backgroundColor: '#2C2C2E', border: '1px solid #48484A' }">
      <!-- Title -->
      <div class="flex items-center justify-between px-5 py-4" :style="{ borderBottom: '1px solid #3A3A3C' }">
        <div class="flex items-center gap-2.5">
          <div class="flex h-7 w-7 items-center justify-center rounded-md" :style="{ backgroundColor: 'rgba(59,130,246,0.18)' }">
            <Download class="h-3.5 w-3.5" :style="{ color: '#3B82F6' }" />
          </div>
          <h2 class="text-sm font-semibold" :style="{ color: '#F5F5F7' }">新建下载</h2>
        </div>
        <button @click="emit('close')" class="rounded p-1 transition-colors" :style="{ color: '#8E8E93' }">
          <X class="h-4 w-4" />
        </button>
      </div>

      <!-- Body -->
      <div class="space-y-4 overflow-y-auto px-5 py-4" style="max-height: 70vh;">
        <!-- URL textarea -->
        <div>
          <div class="mb-1.5 flex items-center justify-between">
            <span class="text-xs font-medium" :style="{ color: '#A1A1A6' }">下载地址 (URL)</span>
            <span v-if="urlCount > 0" class="text-2xs tabular-nums" :style="{ color: '#8E8E93' }">{{ urlCount }} 个链接</span>
          </div>
          <textarea v-model="url" placeholder="https://example.com/file.zip"
            class="w-full resize-none rounded-md px-3 py-2 text-sm outline-none transition-colors"
            :style="{ backgroundColor: '#1C1C1E', border: '1px solid #48484A', color: '#F5F5F7', minHeight: '100px' }" />
          <div class="mt-1.5 flex gap-2">
            <button class="flex items-center gap-1 rounded px-2 py-1 text-2xs transition-colors hover-bg" :style="{ color: '#3B82F6' }">
              <FileDown class="h-3 w-3" /> 种子文件
            </button>
            <button class="flex items-center gap-1 rounded px-2 py-1 text-2xs transition-colors hover-bg" :style="{ color: '#A1A1A6' }">
              <FileText class="h-3 w-3" /> 导入 TXT
            </button>
          </div>
        </div>

        <!-- Save dir -->
        <div>
          <span class="mb-1.5 block text-xs font-medium" :style="{ color: '#A1A1A6' }">保存到</span>
          <div class="flex gap-2">
            <input v-model="saveDir"
              class="flex-1 rounded-md px-3 py-2 text-sm outline-none transition-colors"
              :style="{ backgroundColor: '#1C1C1E', border: '1px solid #48484A', color: '#F5F5F7' }" />
            <button @click="pickSaveDir" class="flex items-center justify-center rounded-md px-3 transition-colors hover-bg-surface2"
              :style="{ border: '1px solid #48484A' }">
              <FolderOpen class="h-4 w-4" :style="{ color: '#A1A1A6' }" />
            </button>
          </div>
        </div>

        <!-- Threads + Rename row -->
        <div class="flex gap-3">
          <div class="flex-1">
            <span class="mb-1.5 block text-xs font-medium" :style="{ color: '#A1A1A6' }">线程数</span>
            <select v-model="segments"
              class="w-full rounded-md px-3 py-2 text-sm outline-none transition-colors"
              :style="{ backgroundColor: '#1C1C1E', border: '1px solid #48484A', color: '#F5F5F7' }">
              <option v-for="opt in segmentOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
            </select>
          </div>
          <div class="flex-[2]">
            <span class="mb-1.5 block text-xs font-medium" :style="{ color: '#A1A1A6' }">重命名</span>
            <input v-model="rename" placeholder="可选"
              class="w-full rounded-md px-3 py-2 text-sm outline-none transition-colors"
              :style="{ backgroundColor: '#1C1C1E', border: '1px solid #48484A', color: '#F5F5F7' }" />
          </div>
        </div>

        <!-- Advanced toggle -->
        <button @click="showAdvanced = !showAdvanced" class="flex items-center gap-1 text-xs font-medium transition-colors" :style="{ color: '#8E8E93' }">
          <ChevronDown class="h-3 w-3" :class="{ 'rotate-180': showAdvanced }" />
          高级选项
        </button>

        <!-- Advanced options -->
        <template v-if="showAdvanced">
          <!-- Proxy -->
          <div>
            <div class="mb-1 flex items-center gap-1.5">
              <span class="text-xs font-medium" :style="{ color: '#A1A1A6' }">代理地址</span>
              <CircleAlert class="h-3 w-3" :style="{ color: '#8E8E93' }" />
            </div>
            <p class="mb-1.5 text-2xs" :style="{ color: '#8E8E93' }">支持 http(s) 代理，格式为 http://地址:端口</p>
            <input v-model="proxyUrl" placeholder="http://127.0.0.1:1080"
              class="w-full rounded-md px-3 py-2 text-sm outline-none transition-colors"
              :style="{ backgroundColor: '#1C1C1E', border: '1px solid #48484A', color: '#F5F5F7' }" />
          </div>

          <!-- User-Agent -->
          <div>
            <span class="mb-1 block text-xs font-medium" :style="{ color: '#A1A1A6' }">User-Agent</span>
            <p class="mb-1.5 text-2xs" :style="{ color: '#8E8E93' }">自定义请求头，覆盖默认 User-Agent</p>
            <input v-model="userAgent" placeholder="Mozilla/5.0 ..."
              class="w-full rounded-md px-3 py-2 text-sm outline-none transition-colors"
              :style="{ backgroundColor: '#1C1C1E', border: '1px solid #48484A', color: '#F5F5F7' }" />
          </div>

          <!-- Cookie -->
          <div>
            <span class="mb-1 block text-xs font-medium" :style="{ color: '#A1A1A6' }">Cookie</span>
            <p class="mb-1.5 text-2xs" :style="{ color: '#8E8E93' }">设置请求 Cookie</p>
            <textarea v-model="cookie" placeholder="name=value; name2=value2"
              class="w-full resize-none rounded-md px-3 py-2 text-sm outline-none transition-colors"
              :style="{ backgroundColor: '#1C1C1E', border: '1px solid #48484A', color: '#F5F5F7', minHeight: '48px' }" />
          </div>

          <!-- Checksum -->
          <div>
            <span class="mb-1 block text-xs font-medium" :style="{ color: '#A1A1A6' }">哈希校验</span>
            <p class="mb-1.5 text-2xs" :style="{ color: '#8E8E93' }">下载完成后校验文件完整性</p>
            <div class="flex gap-2">
              <select v-model="checksumAlgo"
                class="w-28 rounded-md px-3 py-2 text-sm outline-none transition-colors"
                :style="{ backgroundColor: '#1C1C1E', border: '1px solid #48484A', color: '#F5F5F7' }">
                <option v-for="algo in checksumAlgorithms" :key="algo" :value="algo">{{ algo }}</option>
              </select>
              <input v-model="checksum" placeholder="输入哈希值"
                class="flex-1 rounded-md px-3 py-2 text-sm outline-none transition-colors"
                :style="{ backgroundColor: '#1C1C1E', border: '1px solid #48484A', color: '#F5F5F7' }" />
            </div>
          </div>

          <!-- Custom headers -->
          <div>
            <span class="mb-1 block text-xs font-medium" :style="{ color: '#A1A1A6' }">自定义请求头</span>
            <p class="mb-1.5 text-2xs" :style="{ color: '#8E8E93' }">添加额外的 HTTP 请求头</p>
            <div v-for="(row, i) in headerRows" :key="i" class="mb-1.5 flex items-center gap-1.5">
              <input v-model="row.key" placeholder="Key"
                class="flex-[2] rounded-md px-2.5 py-1.5 text-sm outline-none transition-colors"
                :style="{ backgroundColor: '#1C1C1E', border: '1px solid #48484A', color: '#F5F5F7' }" />
              <input v-model="row.value" placeholder="Value"
                class="flex-[3] rounded-md px-2.5 py-1.5 text-sm outline-none transition-colors"
                :style="{ backgroundColor: '#1C1C1E', border: '1px solid #48484A', color: '#F5F5F7' }" />
              <button @click="removeHeader(i)" class="rounded p-1 transition-colors" :style="{ color: '#8E8E93' }">
                <X class="h-3.5 w-3.5" />
              </button>
            </div>
            <button @click="addHeader" class="flex items-center gap-1 rounded px-2 py-1 text-2xs transition-colors hover-bg" :style="{ color: '#3B82F6' }">
              <Plus class="h-3 w-3" /> 添加请求头
            </button>
          </div>
        </template>
      </div>

      <!-- Footer actions -->
      <div class="flex items-center justify-end gap-3 px-5 py-4" :style="{ borderTop: '1px solid #3A3A3C' }">
        <button @click="emit('close')"
          class="rounded-md px-4 py-1.5 text-sm font-medium transition-colors hover-bg-surface2" :style="{ color: '#A1A1A6' }">
          取消
        </button>
        <button @click="submit(true)" :disabled="!url.trim()"
          class="flex items-center gap-1.5 rounded-md px-4 py-1.5 text-sm font-medium transition-colors disabled:opacity-50"
          :style="{ backgroundColor: '#3A3A3C', color: '#A1A1A6' }">
          <Clock class="h-3.5 w-3.5" /> 稍后下载
        </button>
        <button @click="submit(false)" :disabled="!url.trim()"
          class="flex items-center gap-1.5 rounded-md px-4 py-1.5 text-sm font-medium transition-colors disabled:opacity-50"
          :style="{ backgroundColor: '#3B82F6', color: '#fff' }">
          <Download class="h-3.5 w-3.5" /> 开始下载
        </button>
      </div>
    </div>
  </div>
</template>
