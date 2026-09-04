<script setup>
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";

// ==================== 状态管理 ====================
const currentPage = ref('home'); // home | analysis | step1 | step2 | step3 | complete

// PDF 信息
const pdfInfo = ref({
  filename: '',
  pageCount: 0,
  width: 0,
  height: 0,
  paperSize: '',
  path: '',
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
    // 双面打印：进入向导
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
}

function goBack() {
  currentPage.value = 'home';
}

// 打开打印对话框
function openPrintDialog() {
  // 单面打印直接用原文件，双面打印用 secondFile
  const fileToPrint = printMode.value === 'single' ? pdfInfo.value.path : secondFile.value;
  alert(`打印文件: ${fileToPrint}\n\n实际打印功能将在后续实现。`);
  // 打印后进入完成页
  goToComplete();
}

// 格式化页面顺序
function formatOrder(pages) {
  return pages.join(' → ');
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
      <div class="header">
        <button class="btn-back" @click="goBack">← 返回</button>
        <span class="filename">📄 {{ pdfInfo.filename }}</span>
      </div>

      <div class="pdf-info">
        <div class="info-item">📄 {{ pdfInfo.pageCount }} 页</div>
        <div class="info-item">📐 {{ pdfInfo.paperSize }}</div>
      </div>

      <div class="section">
        <div class="section-title">打印模式</div>
        <div class="radio-group">
          <label class="radio-option" :class="{ active: printMode === 'single' }">
            <input type="radio" v-model="printMode" value="single" />
            <span>单面打印</span>
          </label>
          <label class="radio-option" :class="{ active: printMode === 'duplex' }">
            <input type="radio" v-model="printMode" value="duplex" />
            <span>手动双面打印</span>
          </label>
        </div>
        <div v-if="printMode === 'duplex'" class="hint">
          预计需要 {{ duplexPlan.sheetCount }} 张纸
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
</style>
