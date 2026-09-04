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
  // 打印完成后清理临时文件
  cleanupTempFiles();
}

async function cleanupTempFiles() {
  if (pdfInfo.value.path) {
    try {
      await invoke('cmd_cleanup_temp_files', { originalPath: pdfInfo.value.path });
    } catch (e) {
      console.error('清理临时文件失败:', e);
    }
  }
}

function goHome() {
  // 返回首页前清理临时文件
  cleanupTempFiles();
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
        pages: pageSelection.value.selected,
        side: 0
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

// 双面打印第一步：打印第一面
async function printDuplexFirst() {
  try {
    isLoading.value = true;

    // 根据选择的页面提取第一面
    const result = await invoke('cmd_extract_pages', {
      inputPath: pdfInfo.value.path,
      pages: duplexPlan.value.firstPass,
      side: 1
    });

    await invoke('cmd_print_pdf', {
      filePath: result.output_path,
      printerName: selectedPrinter.value
    });

    // 打印完成后自动跳转到步骤2
    goToStep2();
  } catch (e) {
    console.error('打印失败:', e);
    errorMessage.value = `打印失败: ${e}`;
    alert(`打印失败: ${e}`);
  } finally {
    isLoading.value = false;
  }
}

// 双面打印第二步：打印第二面
async function printDuplexSecond() {
  try {
    isLoading.value = true;

    // 根据选择的页面提取第二面
    const result = await invoke('cmd_extract_pages', {
      inputPath: pdfInfo.value.path,
      pages: duplexPlan.value.secondPass,
      side: 2
    });

    await invoke('cmd_print_pdf', {
      filePath: result.output_path,
      printerName: selectedPrinter.value
    });

    // 打印完成后自动跳转到完成页
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
// 进入预览页前保存的选择（用于返回时恢复）
const savedPageSelection = ref([]);

// 跳转到预览页面（用于手动选择页面）
function goToPreview() {
  // 保存当前选择
  savedPageSelection.value = [...pageSelection.value.selected];
  pageSelection.value.thumbnails = {};
  pageSelection.value.loadedPages = 0;
  currentPage.value = 'preview';
}

// 返回分析页面
function goBackToAnalysis() {
  // 恢复之前的页面选择
  pageSelection.value.selected = [...savedPageSelection.value];
  selectedPageCount.value = savedPageSelection.value.length;
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

  // 保持当前选择，返回分析页面
  if (selectedPageCount.value === 1) {
    printMode.value = 'single';
  }
  currentPage.value = 'analysis';
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
      pages: pageSelection.value.selected,
      side: 0
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
      <p class="app-tagline">手动双面打印助手</p>

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
        <div v-else class="hint">
          预计需要 {{ selectedPageCount }} 张纸
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
      </div>

      <div class="pdf-info">
        <div class="info-item">📄 {{ pdfInfo.pageCount }} 页</div>
        <div class="info-item">📐 {{ pdfInfo.paperSize }}</div>
      </div>

      <!-- 快捷操作 -->
      <div class="quick-actions">
        <button class="quick-btn" @click="selectAll">
          <span class="quick-icon">☐</span>
          <span>全选</span>
        </button>
        <button class="quick-btn" @click="selectOdd">
          <span class="quick-icon">1</span>
          <span>奇数页</span>
        </button>
        <button class="quick-btn" @click="selectEven">
          <span class="quick-icon">2</span>
          <span>偶数页</span>
        </button>
        <button class="quick-btn" @click="invertSelection">
          <span class="quick-icon">⇆</span>
          <span>反选</span>
        </button>
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
        </div>
      </div>

      <!-- 底部操作栏 -->
      <div class="preview-footer">
        <div class="selected-info">
          已选 {{ getSelectedCount() }} / {{ pdfInfo.pageCount }} 页
        </div>
        <button class="btn-confirm" @click="printSelected">
          确定
        </button>
      </div>
    </div>

    <!-- ==================== 步骤 1：打印第一面 ==================== -->
    <div v-if="currentPage === 'step1'" class="page step-page">
      <!-- 进度指示器 -->
      <div class="step-indicator-bar">
        <div class="step-dots">
          <div class="step-dot active"></div>
          <div class="step-dot"></div>
          <div class="step-dot"></div>
        </div>
        <div class="step-progress">1 / 3</div>
      </div>

      <div class="step-main">
        <div class="step-illustration">🖨️</div>

        <div class="step-text">
          <h2 class="step-title">打印第一面</h2>
          <p class="step-subtitle">共 {{ selectedPageCount }} 页 · 需要 {{ Math.ceil(selectedPageCount / 2) }} 张纸</p>
        </div>

        <div class="paper-info">
          <div class="paper-icon">📄</div>
          <div class="paper-hint">请将纸张放入打印机</div>
        </div>
      </div>

      <div class="step-actions">
        <button class="btn-primary-action" @click="printDuplexFirst">
          开始打印
        </button>
        <button class="btn-ghost" @click="goHome">
          取消
        </button>
      </div>

      <div class="step-tip">
        💡 打印完成后，系统会引导你翻转纸张继续打印
      </div>
    </div>

    <!-- ==================== 步骤 2：翻转纸张 ==================== -->
    <div v-if="currentPage === 'step2'" class="page step-page">
      <!-- 进度指示器 -->
      <div class="step-indicator-bar">
        <div class="step-dots">
          <div class="step-dot done"></div>
          <div class="step-dot active"></div>
          <div class="step-dot"></div>
        </div>
        <div class="step-progress">2 / 3</div>
      </div>

      <div class="step-main">
        <div class="step-illustration flip-animation">🔄</div>

        <div class="step-text">
          <h2 class="step-title">翻转纸张</h2>
        </div>

        <div class="step2-box">
          <div class="step2-list">
            <div class="step2-item">
              <span class="step2-num">1</span>
              <span>取出纸张</span>
            </div>
            <div class="step2-item">
              <span class="step2-num">2</span>
              <span>翻转（空白朝上）</span>
            </div>
            <div class="step2-item">
              <span class="step2-num">3</span>
              <span>放回纸盒</span>
            </div>
          </div>

          <div class="step2-warning">
            <span class="warning-icon">⚠️</span>
            <span>字向打印机方向</span>
          </div>
        </div>
      </div>

      <div class="step-actions">
        <button class="btn-primary-action" @click="goToStep3">
          已翻转完成
        </button>
        <button class="btn-ghost" @click="goHome">
          取消
        </button>
      </div>
    </div>

    <!-- ==================== 步骤 3：打印第二面 ==================== -->
    <div v-if="currentPage === 'step3'" class="page step-page">
      <!-- 进度指示器 -->
      <div class="step-indicator-bar">
        <div class="step-dots">
          <div class="step-dot done"></div>
          <div class="step-dot done"></div>
          <div class="step-dot active"></div>
        </div>
        <div class="step-progress">3 / 3</div>
      </div>

      <div class="step-main">
        <div class="step-illustration">🖨️</div>

        <div class="step-text">
          <h2 class="step-title">打印第二面</h2>
          <p class="step-subtitle">继续打印剩余页面</p>
        </div>

        <div class="paper-info">
          <div class="paper-icon">📄</div>
          <div class="paper-hint">纸张已准备好，开始打印</div>
        </div>
      </div>

      <div class="step-actions">
        <button class="btn-primary-action" @click="printDuplexSecond">
          开始打印
        </button>
        <button class="btn-ghost" @click="goHome">
          取消
        </button>
      </div>

      <div v-if="selectedPageCount % 2 === 1" class="step-notice">
        <span class="notice-icon">ℹ️</span>
        最后一张纸只有一面有内容，这是正常的
      </div>
    </div>

    <!-- ==================== 完成页 ==================== -->
    <div v-if="currentPage === 'complete'" class="page complete-page">
      <div class="complete-icon">🎉</div>
      <h2 class="complete-title">打印完成</h2>
      <p class="complete-desc">已成功打印 {{ selectedPageCount }} 页</p>
      <div class="complete-stats">
        <span>✨</span>
      </div>
      <button class="btn-primary-action complete-btn" @click="goHome">
        完成
      </button>
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
  background: #FAFAF8;
  color: #2D3436;
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
  font-size: 36px;
  font-weight: 700;
  color: #1B4332;
  margin-bottom: 12px;
  letter-spacing: -0.5px;
}

.app-tagline {
  font-size: 15px;
  color: #6B7280;
  margin-bottom: 40px;
}

.dropzone {
  width: 100%;
  max-width: 420px;
  height: 220px;
  border: 2px dashed #D1D5DB;
  border-radius: 20px;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  cursor: pointer;
  transition: all 0.25s ease;
  background: white;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.04);
}

.dropzone:hover {
  border-color: #E76F51;
  background: #FEF7ED;
  transform: translateY(-2px);
  box-shadow: 0 8px 30px rgba(231, 111, 81, 0.12);
}

.dropzone-icon {
  font-size: 56px;
  margin-bottom: 16px;
}

.dropzone-text {
  color: #2D3436;
  font-size: 17px;
  font-weight: 600;
}

.dropzone-hint {
  color: #9CA3AF;
  font-size: 14px;
  margin-top: 8px;
}

.loading {
  margin-top: 20px;
  color: #E76F51;
  font-size: 14px;
  font-weight: 500;
}

.error {
  margin-top: 20px;
  color: #DC2626;
  font-size: 14px;
  padding: 14px 18px;
  background: #FEF2F2;
  border-radius: 10px;
  border-left: 4px solid #EF4444;
}

/* ==================== 分析页 ==================== */
.analysis-page .btn-primary,
.btn-primary {
  width: 100%;
  margin-top: 16px;
  padding: 18px 24px;
  background: linear-gradient(135deg, #E76F51, #F4A261);
  color: white;
  border: none;
  border-radius: 14px;
  font-size: 18px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.25s ease;
  box-shadow: 0 4px 14px rgba(231, 111, 81, 0.35);
}

.btn-primary:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(231, 111, 81, 0.45);
}

.analysis-page .btn-preview {
  background: #2D6A4F;
  color: white;
  border: none;
  padding: 10px 18px;
  border-radius: 10px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.analysis-page .btn-preview:hover {
  background: #1B4332;
}

/* ==================== 页面头部 ==================== */
.analysis-page .header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 28px;
  padding-top: 8px;
}

.analysis-page .btn-back {
  background: none;
  border: none;
  color: #E76F51;
  font-size: 15px;
  font-weight: 500;
  cursor: pointer;
  padding: 10px 16px;
  border-radius: 10px;
  transition: background 0.2s;
}

.analysis-page .btn-back:hover {
  background: #FEF7ED;
}

.analysis-page .filename {
  color: #6B7280;
  font-size: 14px;
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.analysis-page .pdf-info {
  display: flex;
  gap: 12px;
  margin-bottom: 32px;
}

.analysis-page .info-item {
  background: white;
  padding: 14px 20px;
  border-radius: 12px;
  color: #1B4332;
  font-size: 15px;
  font-weight: 500;
  box-shadow: 0 2px 8px rgba(0,0,0,0.05);
}

.analysis-page .section {
  margin-bottom: 24px;
}

.analysis-page .section-title {
  font-size: 12px;
  color: #9CA3AF;
  margin-bottom: 14px;
  text-transform: uppercase;
  letter-spacing: 1px;
  font-weight: 600;
}

/* ==================== 打印选项 ==================== */
.radio-group {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.radio-option {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 16px 18px;
  background: white;
  border-radius: 14px;
  cursor: pointer;
  transition: all 0.2s;
  border: 2px solid transparent;
  box-shadow: 0 2px 8px rgba(0,0,0,0.04);
}

.radio-option:hover {
  border-color: #F4A261;
}

.radio-option.active {
  border-color: #E76F51;
  background: #FEF7ED;
}

.radio-option input[type="radio"] {
  width: 22px;
  height: 22px;
  accent-color: #E76F51;
}

.radio-option span {
  font-size: 16px;
  color: #2D3436;
  font-weight: 500;
}

.radio-hint {
  font-size: 13px;
  color: #9CA3AF;
  margin-left: auto;
}

.hint {
  margin-top: 12px;
  padding: 12px 16px;
  background: #EFF6FF;
  border-radius: 10px;
  color: #1E40AF;
  font-size: 14px;
}

/* 打印机选择 */
.printer-select {
  width: 100%;
  padding: 14px 16px;
  background: white;
  border: 2px solid #E9ECEF;
  border-radius: 12px;
  font-size: 15px;
  color: #2D3436;
  cursor: pointer;
  transition: all 0.2s;
  box-shadow: 0 2px 8px rgba(0,0,0,0.04);
}

.printer-select:hover {
  border-color: #D1D5DB;
}

.printer-select:focus {
  outline: none;
  border-color: #E76F51;
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

/* ==================== 步骤页（新设计） ==================== */
.step-page {
  background: #FAFAF8;
  min-height: 100vh;
}

/* 进度指示器 */
.step-indicator-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 0;
  margin-bottom: 20px;
}

.step-dots {
  display: flex;
  gap: 12px;
}

.step-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: #E9ECEF;
  transition: all 0.3s ease;
}

.step-dot.active {
  background: #E76F51;
  transform: scale(1.2);
  box-shadow: 0 0 0 4px rgba(231, 111, 81, 0.2);
}

.step-dot.done {
  background: #2D6A4F;
}

.step-progress {
  font-size: 14px;
  color: #6B7280;
  font-weight: 500;
}

/* 主要内容区域 */
.step-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  padding: 20px 0;
}

.step-illustration {
  font-size: 72px;
  margin-bottom: 32px;
}

.step-text {
  margin-bottom: 40px;
}

.step-title {
  font-size: 32px;
  font-weight: 700;
  color: #2D3436;
  margin-bottom: 12px;
}

.step-subtitle {
  font-size: 16px;
  color: #6B7280;
  font-weight: 400;
}

/* 纸张提示 */
.paper-info {
  display: flex;
  align-items: center;
  gap: 12px;
  background: white;
  padding: 16px 24px;
  border-radius: 12px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
}

.paper-icon {
  font-size: 28px;
}

.paper-hint {
  font-size: 15px;
  color: #4B5563;
  font-weight: 500;
}

/* 步骤2列表 */
.step2-box {
  background: white;
  border-radius: 14px;
  padding: 20px 24px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.06);
}

.step2-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 16px;
}

.step2-item {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 15px;
  color: #374151;
  font-weight: 500;
}

.step2-num {
  width: 26px;
  height: 26px;
  background: #E76F51;
  color: white;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  font-weight: 600;
}

.step2-warning {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  background: #FEF3C7;
  border-radius: 8px;
  color: #92400E;
  font-size: 14px;
  font-weight: 600;
}

.warning-icon {
  font-size: 16px;
}

/* 翻转动画 */
.flip-animation {
  animation: flipGentle 2s ease-in-out infinite;
}

@keyframes flipGentle {
  0%, 100% { transform: rotateY(0deg); }
  50% { transform: rotateY(180deg); }
}

/* 操作按钮区域 */
.step-actions {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 0 20px;
  max-width: 360px;
  margin: 0 auto;
  width: 100%;
}

.btn-primary-action {
  width: 100%;
  padding: 18px 24px;
  background: linear-gradient(135deg, #E76F51, #F4A261);
  color: white;
  border: none;
  border-radius: 14px;
  font-size: 18px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.25s ease;
  box-shadow: 0 4px 14px rgba(231, 111, 81, 0.35);
}

.btn-primary-action:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(231, 111, 81, 0.45);
}

.btn-primary-action:active {
  transform: translateY(0);
}

.btn-ghost {
  width: 100%;
  padding: 14px 24px;
  background: transparent;
  color: #9CA3AF;
  border: none;
  border-radius: 10px;
  font-size: 15px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-ghost:hover {
  background: rgba(0, 0, 0, 0.04);
  color: #6B7280;
}

/* 提示文字 */
.step-tip {
  margin-top: 32px;
  padding: 16px 20px;
  background: #FEF7ED;
  border-left: 4px solid #F4A261;
  border-radius: 0 8px 8px 0;
  color: #92400E;
  font-size: 14px;
  line-height: 1.5;
  max-width: 360px;
  margin-left: auto;
  margin-right: auto;
}

/* 通知提示 */
.step-notice {
  margin-top: 24px;
  padding: 14px 18px;
  background: #EFF6FF;
  border-left: 4px solid #3B82F6;
  border-radius: 0 8px 8px 0;
  color: #1E40AF;
  font-size: 14px;
  max-width: 360px;
  margin-left: auto;
  margin-right: auto;
  display: flex;
  align-items: flex-start;
  gap: 10px;
}

.notice-icon {
  font-size: 16px;
  flex-shrink: 0;
}

/* ==================== 旧样式保留兼容 ==================== */
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
  background: #FAFAF8;
}

.complete-icon {
  font-size: 96px;
  margin-bottom: 24px;
  animation: celebrateBounce 0.6s ease-out;
}

@keyframes celebrateBounce {
  0% { transform: scale(0); }
  50% { transform: scale(1.2); }
  100% { transform: scale(1); }
}

.complete-title {
  font-size: 32px;
  font-weight: 700;
  color: #2D3436;
  margin-bottom: 12px;
}

.complete-desc {
  font-size: 16px;
  color: #6B7280;
  margin-bottom: 40px;
}

.complete-stats {
  font-size: 24px;
  margin-bottom: 32px;
}

.complete-btn {
  width: auto;
  padding: 16px 48px;
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
  background: #FAFAF8;
  padding: 0;
}

.preview-page .header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 20px 16px;
  background: white;
  border-bottom: 1px solid #E9ECEF;
  margin-bottom: 0;
}

.preview-page .btn-back {
  background: none;
  border: none;
  color: #E76F51;
  font-size: 15px;
  font-weight: 500;
  cursor: pointer;
  padding: 10px 16px;
  border-radius: 10px;
  transition: background 0.2s;
}

.preview-page .btn-back:hover {
  background: #FEF7ED;
}

.preview-page .filename {
  color: #6B7280;
  font-size: 14px;
  max-width: 180px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.preview-page .pdf-info {
  display: flex;
  gap: 12px;
  padding: 12px 20px;
  background: #FAFAF8;
}

.preview-page .info-item {
  background: white;
  padding: 10px 16px;
  border-radius: 10px;
  color: #1B4332;
  font-size: 14px;
  font-weight: 500;
  box-shadow: 0 1px 3px rgba(0,0,0,0.05);
}

.quick-actions {
  display: flex;
  gap: 10px;
  padding: 16px 20px;
  background: white;
  border-bottom: 1px solid #E9ECEF;
}

.quick-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  background: #F8F9FA;
  border: 1px solid #E9ECEF;
  padding: 8px 14px;
  border-radius: 20px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  color: #4B5563;
}

.quick-btn:hover {
  background: #FEF7ED;
  border-color: #F4A261;
  color: #E76F51;
}

.quick-icon {
  font-size: 14px;
  font-weight: 600;
  width: 18px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: white;
  border-radius: 4px;
  border: 1px solid #E9ECEF;
}

.quick-btn:hover .quick-icon {
  background: #FEF7ED;
  border-color: #F4A261;
}

.page-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(90px, 1fr));
  gap: 10px;
  padding: 16px 20px;
  padding-bottom: 100px;
  background: #FAFAF8;
  max-height: calc(100vh - 180px);
  overflow-y: auto;
}

.page-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 8px 6px;
  background: white;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
  border: 2px solid transparent;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.06);
}

.page-item:hover {
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.1);
}

.page-item.selected {
  border-color: #E76F51;
  background: #FEF7ED;
}

.page-thumb {
  width: 72px;
  height: 115px;
  background: #F8F9FA;
  border-radius: 4px;
  display: flex;
  justify-content: center;
  align-items: center;
  margin-bottom: 4px;
  border: 1px solid #E9ECEF;
}

.page-placeholder {
  font-size: 20px;
  color: #9CA3AF;
  font-weight: 600;
}

.page-number {
  font-size: 12px;
  color: #6B7280;
  font-weight: 500;
}

.page-check {
  font-size: 14px;
  margin-top: 4px;
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
  box-shadow: 0 -2px 10px rgba(0, 0, 0, 0.08);
}

.btn-confirm {
  padding: 12px 32px;
  background: linear-gradient(135deg, #E76F51, #F4A261);
  color: white;
  border: none;
  border-radius: 10px;
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  box-shadow: 0 2px 8px rgba(231, 111, 81, 0.3);
}

.btn-confirm:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(231, 111, 81, 0.4);
}

.selected-info {
  color: #6B7280;
  font-size: 14px;
  font-weight: 500;
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
