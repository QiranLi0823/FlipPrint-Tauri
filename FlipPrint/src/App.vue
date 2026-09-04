<script setup>
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";

// ==================== 状态管理 ====================
const currentPage = ref('home'); // home | analysis | preview | step1 | step2 | step3 | complete

// PDF 信息
const pdfInfo = ref({
  filename: '',
  pageCount: 0,
  width: 0,
  height: 0,
  paperSize: '',
  path: '',
});

// 已选择的页面数
const selectedPageCount = ref(0);

// 页面选择状态
const pageSelection = ref({
  selected: [],        // 已选择的页面数组
  pdfDoc: null,        // PDF 文档对象
  thumbnails: {},      // 缩略图缓存
  loading: false,
  loadedPages: 0,      // 已加载的页数
});

// 打印配置
const printMode = ref('duplex'); // single | duplex
const selectedPrinter = ref('');
const printers = ref([]);

// 双面计划
const duplexPlan = ref({
  firstPass: [],
  secondPass: [],
  sheetCount: 0,
  pageCount: 0,
});

// 生成的 PDF 路径
const firstFile = ref('');
const secondFile = ref('');

// UI 状态
const isLoading = ref(false);
const errorMessage = ref('');

// ==================== 初始化 ====================
onMounted(async () => {
  // 获取打印机列表
  try {
    printers.value = await invoke('cmd_get_printers');
    selectedPrinter.value = await invoke('cmd_get_default_printer');
  } catch (e) {
    console.error('获取打印机失败:', e);
    printers.value = ['默认打印机'];
    selectedPrinter.value = '默认打印机';
  }

  // 监听拖拽事件
  await listen('tauri://drag-drop', async (event) => {
    const files = event.payload.files;
    if (files && files.length > 0) {
      const file = files[0];
      if (file.name.endsWith('.pdf')) {
        await handleFileSelected(file.path);
      }
    }
  });
});

// ==================== 文件处理 ====================
async function selectFile() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'PDF', extensions: ['pdf'] }],
    });

    if (selected) {
      await handleFileSelected(selected);
    }
  } catch (e) {
    console.error('选择文件失败:', e);
    errorMessage.value = '选择文件失败';
  }
}

async function handleFileSelected(filePath) {
  isLoading.value = true;
  errorMessage.value = '';

  try {
    // 调用后端分析 PDF
    const result = await invoke('cmd_generate_duplex', { inputPath: filePath });

    pdfInfo.value = {
      filename: result.info.filename,
      pageCount: result.info.page_count,
      width: result.info.width,
      height: result.info.height,
      paperSize: result.info.paper_size,
      path: filePath,
    };

    // 初始化选择为全选
    pageSelection.value.selected = Array.from(
      { length: result.info.page_count },
      (_, i) => i + 1
    );
    selectedPageCount.value = result.info.page_count;
    hasVisitedPreview.value = false;

    duplexPlan.value = {
      firstPass: result.plan.first_pass,
      secondPass: result.plan.second_pass,
      sheetCount: result.plan.sheet_count,
      pageCount: result.plan.page_count,
    };

    firstFile.value = result.first_file;
    secondFile.value = result.second_file;

    currentPage.value = 'analysis';
  } catch (e) {
    console.error('分析 PDF 失败:', e);
    errorMessage.value = `分析失败: ${e}`;
  } finally {
    isLoading.value = false;
  }
}

// ==================== 页面流转 ====================
function startPrinting() {
  if (printMode.value === 'single') {
    // 单面打印：直接打开打印对话框
    openPrintDialog();
  } else {
    // 双面打印：根据选择页面计算双面计划
    const selected = pageSelection.value.selected;
    const firstPass = [];
    const secondPass = [];

    // 遍历选中的页面（按顺序）
    // 在新提取的PDF中，按顺序位置判断奇偶
    for (let i = 0; i < selected.length; i++) {
      const position = i + 1; // 1-based 位置
      if (position % 2 === 0) {
        // 偶数位置 -> 第一遍（倒序收集）
        firstPass.push(selected[i]);
      } else {
        // 奇数位置 -> 第二遍
        secondPass.push(selected[i]);
      }
    }
    // 第一遍需要倒序
    firstPass.reverse();

    duplexPlan.value = {
      firstPass,
      secondPass,
      sheetCount: Math.ceil(selected.length / 2),
      pageCount: selected.length,
    };

    currentPage.value = 'step1';
  }
}

function goToStep2() {
  currentPage.value = 'step2';
}

function goToStep3() {
  currentPage.value = 'step3';
}

function goToComplete() {
  currentPage.value = 'complete';
}

function goHome() {
  currentPage.value = 'home';
  pdfInfo.value = { filename: '', pageCount: 0, width: 0, height: 0, paperSize: '', path: '' };
  duplexPlan.value = { firstPass: [], secondPass: [], sheetCount: 0, pageCount: 0 };
  hasVisitedPreview.value = false;
  selectedPageCount.value = 0;
}

function goBack() {
  currentPage.value = 'home';
}

// 打开打印对话框
async function openPrintDialog() {
  try {
    isLoading.value = true;

    let fileToPrint;

    if (printMode.value === 'single') {
      // 单面打印：提取选择的页面
      const result = await invoke('cmd_extract_pages', {
        inputPath: pdfInfo.value.path,
        pages: pageSelection.value.selected
      });
      fileToPrint = result.output_path;
    } else {
      // 双面打印用 secondFile
      fileToPrint = secondFile.value;
    }

    await invoke('cmd_print_pdf', {
      filePath: fileToPrint,
      printerName: selectedPrinter.value
    });
    goToComplete();
  } catch (e) {
    console.error('打印失败:', e);
    errorMessage.value = `打印失败: ${e}`;
    alert(`打印失败: ${e}`);
  } finally {
    isLoading.value = false;
  }
}

// 格式化页面顺序
function formatOrder(pages) {
  return pages.join(' → ');
}

// ==================== 预览页面功能 ====================
// 是否已访问过预览页
const hasVisitedPreview = ref(false);

// 跳转到预览页面（用于手动选择页面）
function goToPreview() {
  pageSelection.value.thumbnails = {};
  pageSelection.value.loadedPages = 0;
  currentPage.value = 'preview';
}

// 返回分析页面
function goBackToAnalysis() {
  // 如果选择只有1页，自动切换到单面打印
  if (selectedPageCount.value === 1) {
    printMode.value = 'single';
  }
  currentPage.value = 'analysis';
}

// 页面选择操作
function togglePage(pageNum) {
  const index = pageSelection.value.selected.indexOf(pageNum);
  if (index > -1) {
    pageSelection.value.selected.splice(index, 1);
  } else {
    pageSelection.value.selected.push(pageNum);
    pageSelection.value.selected.sort((a, b) => a - b);
  }
  selectedPageCount.value = pageSelection.value.selected.length;
}

function selectAll() {
  pageSelection.value.selected = Array.from(
    { length: pdfInfo.value.pageCount },
    (_, i) => i + 1
  );
  selectedPageCount.value = pdfInfo.value.pageCount;
}

function selectOdd() {
  const odd = [];
  for (let i = 1; i <= pdfInfo.value.pageCount; i += 2) {
    odd.push(i);
  }
  pageSelection.value.selected = odd;
  selectedPageCount.value = odd.length;
}

function selectEven() {
  const even = [];
  for (let i = 2; i <= pdfInfo.value.pageCount; i += 2) {
    even.push(i);
  }
  pageSelection.value.selected = even;
  selectedPageCount.value = even.length;
}

function invertSelection() {
  const all = Array.from({ length: pdfInfo.value.pageCount }, (_, i) => i + 1);
  pageSelection.value.selected = all.filter(p => !pageSelection.value.selected.includes(p));
  selectedPageCount.value = pageSelection.value.selected.length;
}

// 计算属性：已选页数
function getSelectedCount() {
  return pageSelection.value.selected.length;
}

// 格式化页码显示
function formatPageRange(selected) {
  if (selected.length === 0) return '无';
  if (selected.length === pdfInfo.value.pageCount) return '全部';
  if (selected.length <= 3) return selected.join(', ');
  return `${selected.length} 页`;
}

// 计算选中页数对应的纸张数（双面打印）
function calculateSheetCount(selectedCount) {
  return Math.ceil(selectedCount / 2);
}

// 打印选中的页面
async function printSelected() {
  if (pageSelection.value.selected.length === 0) {
    alert('请至少选择一页');
    return;
  }

  // 返回分析页面继续
  goBackToAnalysis();
}

// 直接开始打印（不返回）
async function startPrintSelected() {
  if (pageSelection.value.selected.length === 0) {
    alert('请至少选择一页');
    return;
  }

  try {
    isLoading.value = true;

    // 调用 Rust 命令生成只包含选中页的 PDF
    const result = await invoke('cmd_extract_pages', {
      inputPath: pdfInfo.value.path,
      pages: pageSelection.value.selected
    });

    // 打印生成的 PDF
    await invoke('cmd_print_pdf', { filePath: result.outputPath });
    goToComplete();
  } catch (e) {
    console.error('打印失败:', e);
    errorMessage.value = `打印失败: ${e}`;
    alert(`打印失败: ${e}`);
  } finally {
    isLoading.value = false;
  }
}
</script>

<template>
  <div class="app">
    <!-- ==================== 首页：拖入 PDF ==================== -->
    <div v-if="currentPage === 'home'" class="page home-page">
      <h1 class="app-title">FlipPrint</h1>

      <div class="dropzone" @click="selectFile">
        <div class="dropzone-icon">📄</div>
        <div class="dropzone-text">点击选择 PDF 文件</div>
        <div class="dropzone-hint">或将文件拖拽到窗口</div>
      </div>

      <div v-if="isLoading" class="loading">
        正在分析 PDF...
      </div>

      <div v-if="errorMessage" class="error">
        {{ errorMessage }}
      </div>
    </div>

    <!-- ==================== 分析页：配置打印 ==================== -->
    <div v-if="currentPage === 'analysis'" class="page analysis-page">
      <div v-if="isLoading" class="loading-overlay">
        <div class="loading-spinner"></div>
        <div class="loading-text">正在处理...</div>
      </div>

      <div class="header">
        <button class="btn-back" @click="goBack">← 返回</button>
        <span class="filename">📄 {{ pdfInfo.filename }}</span>
      </div>

      <div class="pdf-info">
        <div class="info-item">📄 {{ pdfInfo.pageCount }} 页</div>
        <div class="info-item">📐 {{ pdfInfo.paperSize }}</div>
        <button class="btn-preview" @click="goToPreview">
          📑 选择页面
        </button>
      </div>

      <div class="section">
        <div class="section-title">打印模式</div>
        <div class="radio-group">
          <label class="radio-option" :class="{ active: printMode === 'single' }">
            <input type="radio" v-model="printMode" value="single" />
            <span>单面打印</span>
          </label>
          <label
            class="radio-option"
            :class="{ active: printMode === 'duplex', disabled: selectedPageCount < 2 }"
          >
            <input
              type="radio"
              v-model="printMode"
              value="duplex"
              :disabled="selectedPageCount < 2"
            />
            <span>手动双面打印</span>
            <span v-if="selectedPageCount < 2" class="radio-hint">(需要2页以上)</span>
          </label>
        </div>
        <div v-if="printMode === 'duplex'" class="hint">
          预计需要 {{ calculateSheetCount(selectedPageCount) }} 张纸
        </div>
      </div>

      <div class="section">
        <div class="section-title">打印机</div>
        <select v-model="selectedPrinter" class="printer-select">
          <option v-for="printer in printers" :key="printer" :value="printer">
            {{ printer }}
          </option>
        </select>
      </div>

      <button class="btn-primary" @click="startPrinting">
        {{ printMode === 'single' ? '开始打印' : '开始打印向导' }}
      </button>
    </div>

    <!-- ==================== 预览页面：选择要打印的页面 ==================== -->
    <div v-if="currentPage === 'preview'" class="page preview-page">
      <div class="header">
        <button class="btn-back" @click="goBackToAnalysis">← 返回</button>
        <span class="filename">📄 {{ pdfInfo.filename }}</span>
        <span class="selected-count">已选 {{ getSelectedCount() }} / {{ pdfInfo.pageCount }} 页</span>
      </div>

      <!-- 快捷操作 -->
      <div class="quick-actions">
        <button class="quick-btn" @click="selectAll">全选</button>
        <button class="quick-btn" @click="selectOdd">奇数页</button>
        <button class="quick-btn" @click="selectEven">偶数页</button>
        <button class="quick-btn" @click="invertSelection">反选</button>
      </div>

      <!-- 页面网格 -->
      <div class="page-grid">
        <div
          v-for="page in pdfInfo.pageCount"
          :key="page"
          class="page-item"
          :class="{ selected: pageSelection.selected.includes(page) }"
          @click="togglePage(page)"
        >
          <div class="page-thumb">
            <span class="page-placeholder">{{ page }}</span>
          </div>
          <div class="page-number">第 {{ page }} 页</div>
          <div class="page-check">
            {{ pageSelection.selected.includes(page) ? '☑️' : '⬜' }}
          </div>
        </div>
      </div>

      <!-- 底部操作栏 -->
      <div class="preview-footer">
        <div class="selected-info">
          已选择 {{ formatPageRange(pageSelection.selected) }}
        </div>
        <div class="footer-buttons">
          <button class="btn-secondary" @click="printSelected">
            ✓ 确定
          </button>
          <button class="btn-primary" @click="startPrintSelected">
            直接打印 {{ getSelectedCount() }} 页
          </button>
        </div>
      </div>
    </div>

    <!-- ==================== 步骤 1：打印第一面 ==================== -->
    <div v-if="currentPage === 'step1'" class="page step-page">
      <div class="header">
        <span class="step-indicator">步骤 1 / 3</span>
        <span class="filename">📄 {{ pdfInfo.filename }}</span>
      </div>

      <div class="step-content">
        <h2 class="step-title">打印第一面</h2>

        <div class="page-order-box">
          <span class="page-order">{{ formatOrder(duplexPlan.firstPass) }}</span>
        </div>

        <p class="step-desc">请先打印这 {{ duplexPlan.firstPass.length }} 页</p>

        <button class="btn-primary" @click="openPrintDialog">
          打开打印对话框
        </button>
      </div>

      <div class="hint-box">
        💡 打印完成后，翻转纸张重新放入纸盒
      </div>

      <button class="btn-secondary" @click="goToStep2">
        继续
      </button>
    </div>

    <!-- ==================== 步骤 2：翻转纸张 ==================== -->
    <div v-if="currentPage === 'step2'" class="page step-page">
      <div class="header">
        <span class="step-indicator">步骤 2 / 3</span>
        <span class="filename">📄 {{ pdfInfo.filename }}</span>
      </div>

      <div class="step-content">
        <h2 class="step-title">翻转纸张</h2>

        <div class="flip-icon">↻</div>

        <p class="flip-desc">
          将打印好的纸张整体翻转，<br />
          有字的一面朝上，<br />
          然后重新放入纸盒。
        </p>

        <button class="btn-primary" @click="goToStep3">
          继续
        </button>
      </div>
    </div>

    <!-- ==================== 步骤 3：打印第二面 ==================== -->
    <div v-if="currentPage === 'step3'" class="page step-page">
      <div class="header">
        <span class="step-indicator">步骤 3 / 3</span>
        <span class="filename">📄 {{ pdfInfo.filename }}</span>
      </div>

      <div class="step-content">
        <h2 class="step-title">打印第二面</h2>

        <div class="page-order-box">
          <span class="page-order">{{ formatOrder(duplexPlan.secondPass) }}</span>
        </div>

        <p class="step-desc">请打印这 {{ duplexPlan.secondPass.length }} 页</p>

        <button class="btn-primary" @click="openPrintDialog">
          打开打印对话框
        </button>
      </div>

      <div v-if="pdfInfo.pageCount % 2 === 1" class="warning-box">
        ⚠️ 最后一张纸只有一面有内容，这是正常的
      </div>

      <button class="btn-secondary" @click="goToComplete">
        完成
      </button>
    </div>

    <!-- ==================== 完成页 ==================== -->
    <div v-if="currentPage === 'complete'" class="page complete-page">
      <div class="complete-icon">✅</div>
      <h2 class="complete-title">完成！</h2>
      <p class="complete-desc">已成功打印 {{ pdfInfo.pageCount }} 页</p>
      <button class="btn-primary" @click="goHome">完成</button>
    </div>
  </div>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  background: #f5f5f5;
  min-height: 100vh;
}

.app {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}

.page {
  flex: 1;
  padding: 24px;
  display: flex;
  flex-direction: column;
}

/* ==================== 首页 ==================== */
.home-page {
  justify-content: center;
  align-items: center;
}

.app-title {
  font-size: 28px;
  font-weight: 600;
  color: #1f2937;
  margin-bottom: 32px;
}

.dropzone {
  width: 100%;
  max-width: 400px;
  height: 200px;
  border: 2px dashed #d1d5db;
  border-radius: 16px;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  cursor: pointer;
  transition: all 0.2s;
  background: white;
}

.dropzone:hover {
  border-color: #3b82f6;
  background: #f0f9ff;
}

.dropzone-icon {
  font-size: 48px;
  margin-bottom: 12px;
}

.dropzone-text {
  color: #1f2937;
  font-size: 16px;
  font-weight: 500;
}

.dropzone-hint {
  color: #9ca3af;
  font-size: 14px;
  margin-top: 8px;
}

.loading {
  margin-top: 20px;
  color: #3b82f6;
  font-size: 14px;
}

.error {
  margin-top: 20px;
  color: #ef4444;
  font-size: 14px;
  padding: 12px;
  background: #fef2f2;
  border-radius: 8px;
}

/* ==================== 分析页 ==================== */
.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.btn-back {
  background: none;
  border: none;
  color: #3b82f6;
  font-size: 16px;
  cursor: pointer;
}

.filename {
  color: #6b7280;
  font-size: 14px;
}

.pdf-info {
  display: flex;
  gap: 16px;
  margin-bottom: 32px;
}

.info-item {
  background: white;
  padding: 12px 20px;
  border-radius: 8px;
  color: #1f2937;
  font-size: 16px;
}

.section {
  margin-bottom: 24px;
}

.section-title {
  font-size: 14px;
  color: #6b7280;
  margin-bottom: 12px;
}

.radio-group {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.radio-option {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: white;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
  border: 2px solid transparent;
}

.radio-option:hover {
  background: #f3f4f6;
}

.radio-option.active {
  border-color: #3b82f6;
  background: #eff6ff;
}

.radio-option.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.radio-option.disabled:hover {
  background: white;
}

.radio-hint {
  font-size: 12px;
  color: #9ca3af;
  margin-left: 4px;
}

.radio-option input[type="radio"] {
  width: 20px;
  height: 20px;
}

.radio-option span {
  font-size: 16px;
  color: #1f2937;
}

.hint {
  margin-top: 8px;
  font-size: 14px;
  color: #6b7280;
}

.printer-select {
  width: 100%;
  padding: 12px 16px;
  font-size: 16px;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  background: white;
  cursor: pointer;
}

.btn-primary {
  width: 100%;
  padding: 14px;
  background: #3b82f6;
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 16px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.2s;
}

.btn-primary:hover {
  background: #2563eb;
}

.btn-secondary {
  width: 100%;
  padding: 14px;
  background: white;
  color: #3b82f6;
  border: 1px solid #3b82f6;
  border-radius: 8px;
  font-size: 16px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-secondary:hover {
  background: #f0f9ff;
}

/* ==================== 步骤页 ==================== */
.step-indicator {
  font-size: 14px;
  color: #6b7280;
}

.step-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  text-align: center;
}

.step-title {
  font-size: 24px;
  font-weight: 600;
  color: #1f2937;
  margin-bottom: 32px;
}

.page-order-box {
  background: #f3f4f6;
  padding: 24px 48px;
  border-radius: 12px;
  margin-bottom: 16px;
}

.page-order {
  font-size: 32px;
  font-weight: 600;
  color: #1f2937;
  font-family: monospace;
}

.step-desc {
  color: #6b7280;
  font-size: 16px;
  margin-bottom: 32px;
}

.hint-box {
  background: #fef3c7;
  color: #92400e;
  padding: 16px;
  border-radius: 8px;
  text-align: center;
  margin: 24px 0;
  font-size: 14px;
}

.flip-icon {
  font-size: 80px;
  margin-bottom: 24px;
  animation: rotate 2s linear infinite;
}

@keyframes rotate {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.flip-desc {
  color: #6b7280;
  font-size: 16px;
  line-height: 1.6;
  margin-bottom: 32px;
}

.warning-box {
  background: #fef3c7;
  color: #92400e;
  padding: 16px;
  border-radius: 8px;
  text-align: center;
  margin: 24px 0;
  font-size: 14px;
}

/* ==================== 完成页 ==================== */
.complete-page {
  justify-content: center;
  align-items: center;
  text-align: center;
}

.complete-icon {
  font-size: 80px;
  margin-bottom: 24px;
}

.complete-title {
  font-size: 28px;
  font-weight: 600;
  color: #1f2937;
  margin-bottom: 12px;
}

.complete-desc {
  font-size: 16px;
  color: #6b7280;
  margin-bottom: 24px;
}

.complete-stats {
  font-size: 18px;
  color: #1f2937;
  margin-bottom: 32px;
}

/* ==================== PDF 预览样式 ==================== */
.btn-preview {
  background: #10b981;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  transition: background 0.2s;
}

.btn-preview:hover {
  background: #059669;
}

.pdf-preview-modal {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.8);
  z-index: 1000;
  display: flex;
  flex-direction: column;
}

.preview-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  background: #1f2937;
  color: white;
}

.preview-title {
  font-size: 16px;
  font-weight: 500;
}

.preview-close {
  background: none;
  border: none;
  color: white;
  font-size: 24px;
  cursor: pointer;
  padding: 4px 8px;
}

.preview-toolbar {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 12px 20px;
  background: #374151;
  color: white;
}

.preview-btn {
  background: #4b5563;
  border: none;
  color: white;
  padding: 6px 12px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
}

.preview-btn:hover:not(:disabled) {
  background: #6b7280;
}

.preview-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.zoom-level {
  font-size: 14px;
  min-width: 50px;
  text-align: center;
}

.page-nav {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-left: auto;
}

.page-info {
  color: white;
  font-size: 14px;
  min-width: 80px;
  text-align: center;
}

.preview-container {
  flex: 1;
  overflow: auto;
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding: 20px;
  background: #111827;
}

.preview-canvas-container {
  background: #f0f0f0;
  display: flex;
  justify-content: center;
  padding: 20px;
}

.preview-canvas {
  display: block;
  background: white;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
}

.preview-iframe-container {
  width: 100%;
  height: 100%;
}

.preview-iframe {
  width: 100%;
  height: 100%;
  min-height: 80vh;
  border: none;
  background: white;
}

.preview-object {
  width: 100%;
  height: 100%;
  min-height: 80vh;
  border: none;
  background: white;
}

.preview-loading {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 200px;
  color: #9ca3af;
  font-size: 16px;
}

/* ==================== 预览页面样式 ==================== */
.preview-page {
  padding: 16px;
}

.preview-page .header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  flex-wrap: wrap;
  gap: 8px;
}

.selected-count {
  background: #3b82f6;
  color: white;
  padding: 4px 12px;
  border-radius: 16px;
  font-size: 13px;
  font-weight: 500;
}

.quick-actions {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
  flex-wrap: wrap;
}

.quick-btn {
  background: #f3f4f6;
  border: 1px solid #e5e7eb;
  padding: 6px 12px;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.quick-btn:hover {
  background: #e5e7eb;
}

.page-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
  gap: 12px;
  max-height: 55vh;
  overflow-y: auto;
  padding: 4px;
  background: #f9fafb;
  border-radius: 8px;
}

.page-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 8px;
  background: white;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
  border: 2px solid transparent;
}

.page-item:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.page-item.selected {
  border-color: #3b82f6;
  background: #eff6ff;
}

.page-thumb {
  width: 80px;
  height: 110px;
  background: #f3f4f6;
  border-radius: 4px;
  display: flex;
  justify-content: center;
  align-items: center;
  margin-bottom: 4px;
}

.page-placeholder {
  font-size: 24px;
  color: #9ca3af;
  font-weight: 600;
}

.page-number {
  font-size: 12px;
  color: #6b7280;
  margin-bottom: 2px;
}

.page-check {
  font-size: 16px;
}

.preview-footer {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  background: white;
  padding: 16px 24px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  box-shadow: 0 -2px 10px rgba(0, 0, 0, 0.1);
}

.footer-buttons {
  display: flex;
  gap: 12px;
}

.footer-buttons .btn-primary,
.footer-buttons .btn-secondary {
  width: auto;
  padding: 12px 20px;
}

.preview-footer .btn-primary {
  width: auto;
  padding: 12px 24px;
}

.selected-info {
  color: #6b7280;
  font-size: 14px;
}

/* ==================== 加载动画 ==================== */
.loading-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255, 255, 255, 0.9);
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  z-index: 9999;
}

.loading-spinner {
  width: 50px;
  height: 50px;
  border: 4px solid #e5e7eb;
  border-top-color: #3b82f6;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.loading-text {
  margin-top: 16px;
  color: #6b7280;
  font-size: 14px;
}
</style>
