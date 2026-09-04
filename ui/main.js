// ==========================================================================
// FanRGB Studio - Advanced Multi-Mode Controller
// Studio Palette · Systematic RGB Spectrum Test · Decoupled White Calibration
// ==========================================================================

// Global state
let currentColor = { r: 255, g: 0, b: 0, hex: "#ff0000", name: "Pure Red" };
let currentMode = "static"; // "static", "breathing", "meteor", "flashing", "off"

// --------------------------------------------------------------------------
// Robust Tauri v2 IPC Bridge
// --------------------------------------------------------------------------
async function invokeCommand(cmd, args = {}) {
  let fn = null;
  if (window.__TAURI__?.core?.invoke) {
    fn = window.__TAURI__.core.invoke;
  } else if (window.__TAURI_INTERNALS__?.invoke) {
    fn = window.__TAURI_INTERNALS__.invoke;
  } else if (window.__TAURI__?.tauri?.invoke) {
    fn = window.__TAURI__.tauri.invoke;
  } else if (window.invoke) {
    fn = window.invoke;
  }

  if (!fn) {
    const errorMsg = "Tauri IPC bridge is not ready. Launch through desktop app.";
    console.error(errorMsg, cmd, args);
    showToast(`Error: ${errorMsg}`);
    throw new Error(errorMsg);
  }

  try {
    return await fn(cmd, args);
  } catch (err) {
    console.error(`[IPC Error] ${cmd}:`, err);
    showToast(`Hardware error: ${err}`);
    throw err;
  }
}

// --------------------------------------------------------------------------
// Transitions.dev: Toast Open / Close Orchestration
// --------------------------------------------------------------------------
let toastTimer = null;
function showToast(msg) {
  const toast = document.getElementById("toast-container");
  const text = document.getElementById("toast-text");
  if (!toast || !text) return;

  text.innerText = msg;
  toast.classList.add("is-open");
  toast.setAttribute("data-open", "true");

  if (toastTimer) clearTimeout(toastTimer);
  toastTimer = setTimeout(() => {
    toast.classList.remove("is-open");
    toast.setAttribute("data-open", "false");
  }, 2400);
}

// --------------------------------------------------------------------------
// Transitions.dev: Universal Sliding Tabs Helper
// --------------------------------------------------------------------------
function setupSlidingTabs(tabsContainer, pillSelector, tabSelector, onSelect) {
  const pill = tabsContainer.querySelector(pillSelector);
  const tabs = [...tabsContainer.querySelectorAll(tabSelector)];

  function move(tab, animate = true) {
    if (!pill || !tab) return;
    if (!animate) pill.style.transition = "none";
    pill.style.transform = `translateX(${tab.offsetLeft}px)`;
    pill.style.width = `${tab.offsetWidth}px`;
    if (!animate) {
      pill.offsetHeight; // reflow
      pill.style.transition = "";
    }
  }

  tabs.forEach((tab) => {
    tab.addEventListener("click", () => {
      tabs.forEach((t) => t.setAttribute("aria-selected", "false"));
      tab.setAttribute("aria-selected", "true");
      move(tab, true);
      if (onSelect) onSelect(tab);
    });
  });

  return {
    move,
    init: () => {
      const active = tabsContainer.querySelector(`${tabSelector}[aria-selected="true"]`) || tabs[0];
      if (active) move(active, false);
    },
  };
}

// --------------------------------------------------------------------------
// Primary View Navigation
// --------------------------------------------------------------------------
const viewTabsNav = document.querySelector(".view-tabs");
let viewTabsController = null;

if (viewTabsNav) {
  viewTabsController = setupSlidingTabs(viewTabsNav, ".view-pill", ".view-tab", (tab) => {
    const targetView = tab.getAttribute("data-view");
    document.querySelectorAll(".view-section").forEach((sec) => sec.classList.remove("active"));
    const activeSec = document.getElementById(`view-${targetView}`);
    if (activeSec) activeSec.classList.add("active");
  });
}

// --------------------------------------------------------------------------
// Color Utilities
// --------------------------------------------------------------------------
function componentToHex(c) {
  const hex = Math.max(0, Math.min(255, Math.round(c))).toString(16);
  return hex.length === 1 ? "0" + hex : hex;
}

function rgbToHex(r, g, b) {
  return "#" + componentToHex(r) + componentToHex(g) + componentToHex(b);
}

function hexToRgb(hex) {
  const clean = hex.replace("#", "");
  const num = parseInt(clean, 16);
  return {
    r: (num >> 16) & 255,
    g: (num >> 8) & 255,
    b: num & 255,
  };
}

// --------------------------------------------------------------------------
// VIEW 1: Studio Palette & Effects
// --------------------------------------------------------------------------
const effectTabsBar = document.querySelector(".effect-tabs");
let effectTabsController = null;

if (effectTabsBar) {
  effectTabsController = setupSlidingTabs(effectTabsBar, ".effect-pill", ".effect-tab", async (tab) => {
    const mode = tab.getAttribute("data-mode") || "static";
    currentMode = mode;
    const effectTag = document.getElementById("active-effect-tag");
    if (effectTag) effectTag.innerText = mode.toUpperCase();
    await sendHardwareUpdate();
  });
}

const sliderR = document.getElementById("slider-r");
const sliderG = document.getElementById("slider-g");
const sliderB = document.getElementById("slider-b");
const numR = document.getElementById("num-r");
const numG = document.getElementById("num-g");
const numB = document.getElementById("num-b");
const rgbSummary = document.getElementById("rgb-summary");
const customPicker = document.getElementById("custom-picker");
const customHex = document.getElementById("custom-hex");
const colorButtons = document.querySelectorAll(".color-card");

function syncStudioControls(source = "all") {
  const { r, g, b, hex } = currentColor;

  if (source !== "sliders") {
    if (sliderR) sliderR.value = r;
    if (sliderG) sliderG.value = g;
    if (sliderB) sliderB.value = b;
  }

  if (source !== "numbers") {
    if (numR) numR.value = r;
    if (numG) numG.value = g;
    if (numB) numB.value = b;
  }

  if (source !== "picker" && customPicker) customPicker.value = hex;
  if (source !== "hex" && customHex) customHex.value = hex.toUpperCase();
  if (rgbSummary) rgbSummary.innerText = `RGB: ${r}, ${g}, ${b}`;

  updateStudioOrbAndTelemetry();
}

function updateStudioOrbAndTelemetry() {
  const isOff = currentMode === "off";
  const orb = document.getElementById("active-orb");
  if (orb) {
    orb.style.setProperty("--glow-color", isOff ? "#18181b" : currentColor.hex);
    if (currentMode === "breathing") {
      orb.classList.add("orb-breathing");
    } else {
      orb.classList.remove("orb-breathing");
    }
  }

  const stateStr = isOff
    ? "System Lighting Powered Off"
    : `${currentColor.name} · ${currentMode.toUpperCase()} (${currentColor.r}, ${currentColor.g}, ${currentColor.b})`;

  const shimmer = document.getElementById("active-shimmer");
  if (shimmer) {
    shimmer.setAttribute("data-text", stateStr);
    shimmer.innerText = stateStr;
  }
}

let studioThrottleTimer = null;
function handleStudioSliderChange(source) {
  const r = parseInt(sliderR?.value, 10) || 0;
  const g = parseInt(sliderG?.value, 10) || 0;
  const b = parseInt(sliderB?.value, 10) || 0;
  const hex = rgbToHex(r, g, b);

  currentColor = { r, g, b, hex, name: "Custom" };
  colorButtons.forEach((btn) => btn.classList.remove("active"));
  syncStudioControls(source);

  if (studioThrottleTimer) clearTimeout(studioThrottleTimer);
  studioThrottleTimer = setTimeout(() => sendHardwareUpdate(), 40);
}

[sliderR, sliderG, sliderB].forEach((slider) => {
  if (!slider) return;
  slider.addEventListener("input", () => handleStudioSliderChange("sliders"));
  slider.addEventListener("change", () => sendHardwareUpdate());
});

[numR, numG, numB].forEach((numInput) => {
  if (!numInput) return;
  numInput.addEventListener("input", () => {
    const r = Math.min(255, Math.max(0, parseInt(numR?.value, 10) || 0));
    const g = Math.min(255, Math.max(0, parseInt(numG?.value, 10) || 0));
    const b = Math.min(255, Math.max(0, parseInt(numB?.value, 10) || 0));

    if (sliderR) sliderR.value = r;
    if (sliderG) sliderG.value = g;
    if (sliderB) sliderB.value = b;

    handleStudioSliderChange("numbers");
  });
});

colorButtons.forEach((btn) => {
  btn.addEventListener("click", async () => {
    colorButtons.forEach((b) => b.classList.remove("active"));
    btn.classList.add("active");

    const hex = btn.getAttribute("data-hex") || "#ff0000";
    const name = btn.getAttribute("data-name") || "Custom";
    const rgb = hexToRgb(hex);

    currentColor = { r: rgb.r, g: rgb.g, b: rgb.b, hex, name };
    syncStudioControls("preset");
    await sendHardwareUpdate();
  });
});

if (customPicker) {
  customPicker.addEventListener("input", (e) => {
    const hex = e.target.value;
    const rgb = hexToRgb(hex);
    currentColor = { r: rgb.r, g: rgb.g, b: rgb.b, hex, name: `Custom (${hex.toUpperCase()})` };
    colorButtons.forEach((b) => b.classList.remove("active"));
    syncStudioControls("picker");

    if (studioThrottleTimer) clearTimeout(studioThrottleTimer);
    studioThrottleTimer = setTimeout(() => sendHardwareUpdate(), 40);
  });
}

if (customHex) {
  customHex.addEventListener("change", async (e) => {
    let hex = e.target.value.trim();
    if (!hex.startsWith("#")) hex = `#${hex}`;
    if (/^#[0-9A-Fa-f]{6}$/.test(hex)) {
      const rgb = hexToRgb(hex);
      currentColor = { r: rgb.r, g: rgb.g, b: rgb.b, hex, name: `Custom (${hex.toUpperCase()})` };
      colorButtons.forEach((b) => b.classList.remove("active"));
      syncStudioControls("hex");
      await sendHardwareUpdate();
    }
  });
}

async function sendHardwareUpdate() {
  const isOff = currentMode === "off";
  const r = isOff ? 0 : currentColor.r;
  const g = isOff ? 0 : currentColor.g;
  const b = isOff ? 0 : currentColor.b;

  updateStudioOrbAndTelemetry();

  const stateStr = isOff
    ? "System Lighting Powered Off"
    : `${currentColor.name} · ${currentMode.toUpperCase()} (${r}, ${g}, ${b})`;

  showToast(`Applying ${stateStr}...`);

  try {
    await invokeCommand("apply_lighting", { r, g, b, mode: currentMode });
    showToast(`Active: ${stateStr}`);
  } catch (err) {
    showToast(`Error: ${err}`);
  }
}

// --------------------------------------------------------------------------
// VIEW 2: Systematic RGB Color Testing Mode
// --------------------------------------------------------------------------
const testState = {
  isRunning: false,
  isPaused: false,
  step: 15,
  delayMs: 250,
  currentIndex: 0,
  values: [],
  totalCombos: 0,
  timerId: null,
};

function computeSpectrumValues(step) {
  const vals = [];
  for (let v = 0; v <= 255; v += step) {
    vals.push(v);
  }
  return vals;
}

function recalculateTestSpectrum() {
  testState.values = computeSpectrumValues(testState.step);
  const n = testState.values.length;
  testState.totalCombos = n * n * n;

  const countTag = document.getElementById("combos-count-tag");
  if (countTag) {
    countTag.innerText = `${testState.totalCombos.toLocaleString()} Combinations`;
  }

  updateTestMonitorUI(testState.currentIndex);
}

function updateTestMonitorUI(idx) {
  const n = testState.values.length;
  if (n === 0) return;

  const clampedIdx = Math.max(0, Math.min(idx, testState.totalCombos - 1));
  const rIdx = Math.floor(clampedIdx / (n * n));
  const gIdx = Math.floor((clampedIdx % (n * n)) / n);
  const bIdx = clampedIdx % n;

  const r = testState.values[rIdx] ?? 0;
  const g = testState.values[gIdx] ?? 0;
  const b = testState.values[bIdx] ?? 0;
  const hex = rgbToHex(r, g, b);

  const swatch = document.getElementById("test-swatch");
  if (swatch) swatch.style.setProperty("--swatch-c", hex);

  const hexReadout = document.getElementById("test-readout-hex");
  if (hexReadout) hexReadout.innerText = hex.toUpperCase();

  const pillR = document.getElementById("test-pill-r");
  const pillG = document.getElementById("test-pill-g");
  const pillB = document.getElementById("test-pill-b");
  if (pillR) pillR.innerText = `R: ${r}`;
  if (pillG) pillG.innerText = `G: ${g}`;
  if (pillB) pillB.innerText = `B: ${b}`;

  // Required exact progress string
  const mainReadout = document.getElementById("test-main-readout");
  if (mainReadout) {
    mainReadout.innerText = `RGB: ${r}, ${g}, ${b} | HEX: ${hex.toUpperCase()} | ${(clampedIdx + 1).toLocaleString()} / ${testState.totalCombos.toLocaleString()}`;
  }

  // Progress Bar & Percentage
  const pct = testState.totalCombos > 0 ? ((clampedIdx + 1) / testState.totalCombos) * 100 : 0;
  const progressBar = document.getElementById("test-progress-bar");
  if (progressBar) progressBar.style.width = `${pct.toFixed(2)}%`;

  const pctLabel = document.getElementById("test-percent-label");
  if (pctLabel) pctLabel.innerText = `${pct.toFixed(2)}%`;

  const ratioLabel = document.getElementById("test-ratio-label");
  if (ratioLabel) {
    ratioLabel.innerText = `${(clampedIdx + 1).toLocaleString()} of ${testState.totalCombos.toLocaleString()} colors evaluated`;
  }

  // ETA calculation
  const etaText = document.getElementById("test-eta-text");
  if (etaText) {
    if (!testState.isRunning && !testState.isPaused) {
      etaText.innerText = `Estimated Time: ~${formatDuration(((testState.totalCombos - clampedIdx) * testState.delayMs) / 1000)}`;
    } else if (testState.isPaused) {
      etaText.innerText = "Estimated Time: Paused";
    } else {
      const remainingSecs = Math.max(0, ((testState.totalCombos - (clampedIdx + 1)) * testState.delayMs) / 1000);
      etaText.innerText = `Estimated Time Remaining: ~${formatDuration(remainingSecs)}`;
    }
  }

  return { r, g, b, hex };
}

function formatDuration(seconds) {
  seconds = Math.round(seconds);
  if (seconds < 60) return `${seconds}s`;
  const mins = Math.floor(seconds / 60);
  const remSecs = seconds % 60;
  if (mins < 60) return `${mins}m ${remSecs}s`;
  const hours = Math.floor(mins / 60);
  const remMins = mins % 60;
  return `${hours}h ${remMins}m`;
}

// Step Chips
const stepChips = document.querySelectorAll("#step-chips .chip-btn");
const testStepInput = document.getElementById("test-step-input");

stepChips.forEach((chip) => {
  chip.addEventListener("click", () => {
    stepChips.forEach((c) => c.classList.remove("active"));
    chip.classList.add("active");
    const step = parseInt(chip.getAttribute("data-step"), 10);
    testState.step = step;
    if (testStepInput) testStepInput.value = step;
    testState.currentIndex = 0;
    recalculateTestSpectrum();
  });
});

if (testStepInput) {
  testStepInput.addEventListener("change", (e) => {
    let step = parseInt(e.target.value, 10);
    if (isNaN(step) || step < 1) step = 1;
    if (step > 128) step = 128;
    testStepInput.value = step;
    testState.step = step;

    stepChips.forEach((c) => {
      c.classList.toggle("active", c.getAttribute("data-step") === String(step));
    });

    testState.currentIndex = 0;
    recalculateTestSpectrum();
  });
}

// Delay Chips with Safety Guard (min 50ms)
const delayChips = document.querySelectorAll("#delay-chips .chip-btn");
const testDelayInput = document.getElementById("test-delay-input");

delayChips.forEach((chip) => {
  chip.addEventListener("click", () => {
    delayChips.forEach((c) => c.classList.remove("active"));
    chip.classList.add("active");
    const delay = parseInt(chip.getAttribute("data-delay"), 10);
    testState.delayMs = Math.max(50, delay);
    if (testDelayInput) testDelayInput.value = testState.delayMs;
  });
});

if (testDelayInput) {
  testDelayInput.addEventListener("change", (e) => {
    let delay = parseInt(e.target.value, 10);
    if (isNaN(delay) || delay < 50) {
      delay = 50;
      showToast("Safety limit applied: Minimum interval is 50ms.");
    }
    testDelayInput.value = delay;
    testState.delayMs = delay;

    delayChips.forEach((c) => {
      c.classList.toggle("active", c.getAttribute("data-delay") === String(delay));
    });
  });
}

// Checkpoint Storage
const CHECKPOINT_KEY = "fanrgb_color_test_checkpoint";

function saveCheckpoint(index) {
  const current = updateTestMonitorUI(index);
  const data = {
    index,
    step: testState.step,
    r: current.r,
    g: current.g,
    b: current.b,
    hex: current.hex,
    totalCombos: testState.totalCombos,
    savedAt: new Date().toISOString(),
  };
  localStorage.setItem(CHECKPOINT_KEY, JSON.stringify(data));
  renderSavedCheckpointBox();
}

function loadCheckpoint() {
  try {
    const raw = localStorage.getItem(CHECKPOINT_KEY);
    if (!raw) return null;
    return JSON.parse(raw);
  } catch (e) {
    return null;
  }
}

function renderSavedCheckpointBox() {
  const box = document.getElementById("saved-state-box");
  const text = document.getElementById("saved-state-text");
  const cp = loadCheckpoint();

  if (cp && box && text) {
    box.style.display = "flex";
    text.innerText = `RGB(${cp.r}, ${cp.g}, ${cp.b}) · Index ${(cp.index + 1).toLocaleString()} / ${cp.totalCombos.toLocaleString()}`;
  } else if (box) {
    box.style.display = "none";
  }
}

const btnClearSave = document.getElementById("btn-clear-save");
if (btnClearSave) {
  btnClearSave.addEventListener("click", () => {
    localStorage.removeItem(CHECKPOINT_KEY);
    renderSavedCheckpointBox();
    showToast("Color test checkpoint cleared.");
  });
}

// Test Mode Controls: Start, Pause, Resume, Stop, Reset
const btnStart = document.getElementById("btn-test-start");
const btnPause = document.getElementById("btn-test-pause");
const btnResume = document.getElementById("btn-test-resume");
const btnStop = document.getElementById("btn-test-stop");
const btnReset = document.getElementById("btn-test-reset");
const testBadge = document.getElementById("test-status-badge");

function setTestBadgeStatus(status) {
  if (!testBadge) return;
  testBadge.className = `test-badge badge-${status.toLowerCase()}`;
  testBadge.innerText = `STATUS: ${status.toUpperCase()}`;
}

async function runColorTestLoop() {
  while (testState.isRunning) {
    if (testState.isPaused) {
      await new Promise((res) => setTimeout(res, 100));
      continue;
    }

    if (testState.currentIndex >= testState.totalCombos) {
      testState.isRunning = false;
      setTestBadgeStatus("completed");
      showToast("Color test completed all combinations!");
      btnStart.disabled = false;
      btnPause.disabled = true;
      btnResume.style.display = "none";
      btnPause.style.display = "inline-flex";
      btnStop.disabled = true;
      break;
    }

    const { r, g, b } = updateTestMonitorUI(testState.currentIndex);

    try {
      await invokeCommand("apply_lighting", { r, g, b, mode: "static" });
    } catch (e) {
      console.warn("Hardware update error during test tick:", e);
    }

    testState.currentIndex++;

    // Safe configurable delay between steps (minimum 50ms)
    const effectiveDelay = Math.max(50, testState.delayMs);
    await new Promise((res) => setTimeout(res, effectiveDelay));
  }
}

if (btnStart) {
  btnStart.addEventListener("click", async () => {
    const cp = loadCheckpoint();
    if (cp && testState.currentIndex === 0 && cp.step === testState.step) {
      testState.currentIndex = cp.index;
      showToast(`Resuming from saved checkpoint at index ${cp.index.toLocaleString()}.`);
    }

    testState.isRunning = true;
    testState.isPaused = false;
    setTestBadgeStatus("running");

    btnStart.disabled = true;
    btnPause.disabled = false;
    btnPause.style.display = "inline-flex";
    btnResume.style.display = "none";
    btnStop.disabled = false;

    runColorTestLoop();
  });
}

if (btnPause) {
  btnPause.addEventListener("click", () => {
    testState.isPaused = true;
    setTestBadgeStatus("paused");
    btnPause.style.display = "none";
    btnResume.style.display = "inline-flex";
    showToast("Color test sequence paused.");
  });
}

if (btnResume) {
  btnResume.addEventListener("click", () => {
    testState.isPaused = false;
    setTestBadgeStatus("running");
    btnResume.style.display = "none";
    btnPause.style.display = "inline-flex";
    showToast("Resuming color test sequence.");
  });
}

if (btnStop) {
  btnStop.addEventListener("click", () => {
    testState.isRunning = false;
    testState.isPaused = false;
    saveCheckpoint(Math.max(0, testState.currentIndex - 1));
    setTestBadgeStatus("idle");

    btnStart.disabled = false;
    btnPause.disabled = true;
    btnResume.style.display = "none";
    btnPause.style.display = "inline-flex";
    btnStop.disabled = true;

    showToast("Test stopped. Checkpoint safely saved.");
  });
}

if (btnReset) {
  btnReset.addEventListener("click", () => {
    testState.isRunning = false;
    testState.isPaused = false;
    testState.currentIndex = 0;
    localStorage.removeItem(CHECKPOINT_KEY);
    renderSavedCheckpointBox();
    setTestBadgeStatus("idle");

    btnStart.disabled = false;
    btnPause.disabled = true;
    btnResume.style.display = "none";
    btnPause.style.display = "inline-flex";
    btnStop.disabled = true;

    updateTestMonitorUI(0);
    showToast("Test index reset to 0.");
  });
}

// --------------------------------------------------------------------------
// VIEW 3: Independent White Point Calibration
// --------------------------------------------------------------------------
const whiteState = {
  r: 255,
  g: 245,
  b: 235, // default D65 Daylight
};

const calibSliderR = document.getElementById("calib-slider-r");
const calibSliderG = document.getElementById("calib-slider-g");
const calibSliderB = document.getElementById("calib-slider-b");
const calibNumR = document.getElementById("calib-num-r");
const calibNumG = document.getElementById("calib-num-g");
const calibNumB = document.getElementById("calib-num-b");
const whiteOrb = document.getElementById("white-orb");
const whiteHexTag = document.getElementById("white-hex-tag");
const whiteRgbString = document.getElementById("white-rgb-string");
const ratioR = document.getElementById("ratio-r");
const ratioG = document.getElementById("ratio-g");
const ratioB = document.getElementById("ratio-b");
const btnSaveWhite = document.getElementById("btn-save-white");
const btnApplyWhite = document.getElementById("btn-apply-white");
const btnLoadSavedWhite = document.getElementById("btn-load-saved-white");
const calibPresetBtns = document.querySelectorAll(".calib-preset-btn");

const WHITE_CALIB_STORAGE_KEY = "fanrgb_calibrated_white";

function syncWhiteCalibrationUI(source = "all") {
  const { r, g, b } = whiteState;
  const hex = rgbToHex(r, g, b);

  if (source !== "sliders") {
    if (calibSliderR) calibSliderR.value = r;
    if (calibSliderG) calibSliderG.value = g;
    if (calibSliderB) calibSliderB.value = b;
  }

  if (source !== "numbers") {
    if (calibNumR) calibNumR.value = r;
    if (calibNumG) calibNumG.value = g;
    if (calibNumB) calibNumB.value = b;
  }

  if (whiteOrb) whiteOrb.style.setProperty("--glow-color", hex);
  if (whiteHexTag) whiteHexTag.innerText = hex.toUpperCase();
  if (whiteRgbString) whiteRgbString.innerText = `R: ${r} | G: ${g} | B: ${b}`;

  // Normalized channel ratio calculations
  const sum = r + g + b || 1;
  const pr = ((r / sum) * 100).toFixed(1);
  const pg = ((g / sum) * 100).toFixed(1);
  const pb = ((b / sum) * 100).toFixed(1);

  if (ratioR) ratioR.innerText = `R: ${pr}%`;
  if (ratioG) ratioG.innerText = `G: ${pg}%`;
  if (ratioB) ratioB.innerText = `B: ${pb}%`;
}

let whiteThrottleTimer = null;
function handleWhiteChange(source) {
  const r = parseInt(calibSliderR?.value, 10) || 0;
  const g = parseInt(calibSliderG?.value, 10) || 0;
  const b = parseInt(calibSliderB?.value, 10) || 0;

  whiteState.r = r;
  whiteState.g = g;
  whiteState.b = b;

  calibPresetBtns.forEach((b) => b.classList.remove("active"));
  syncWhiteCalibrationUI(source);

  if (whiteThrottleTimer) clearTimeout(whiteThrottleTimer);
  whiteThrottleTimer = setTimeout(() => applyWhiteToHardware(), 50);
}

[calibSliderR, calibSliderG, calibSliderB].forEach((slider) => {
  if (!slider) return;
  slider.addEventListener("input", () => handleWhiteChange("sliders"));
  slider.addEventListener("change", () => applyWhiteToHardware());
});

[calibNumR, calibNumG, calibNumB].forEach((num) => {
  if (!num) return;
  num.addEventListener("input", () => {
    const r = Math.min(255, Math.max(0, parseInt(calibNumR?.value, 10) || 0));
    const g = Math.min(255, Math.max(0, parseInt(calibNumG?.value, 10) || 0));
    const b = Math.min(255, Math.max(0, parseInt(calibNumB?.value, 10) || 0));

    if (calibSliderR) calibSliderR.value = r;
    if (calibSliderG) calibSliderG.value = g;
    if (calibSliderB) calibSliderB.value = b;

    handleWhiteChange("numbers");
  });
});

calibPresetBtns.forEach((btn) => {
  btn.addEventListener("click", async () => {
    calibPresetBtns.forEach((b) => b.classList.remove("active"));
    btn.classList.add("active");

    whiteState.r = parseInt(btn.getAttribute("data-r"), 10);
    whiteState.g = parseInt(btn.getAttribute("data-g"), 10);
    whiteState.b = parseInt(btn.getAttribute("data-b"), 10);

    syncWhiteCalibrationUI("preset");
    await applyWhiteToHardware();
  });
});

async function applyWhiteToHardware() {
  try {
    await invokeCommand("apply_lighting", {
      r: whiteState.r,
      g: whiteState.g,
      b: whiteState.b,
      mode: "static",
    });
    showToast(`White calibrated: RGB(${whiteState.r}, ${whiteState.g}, ${whiteState.b})`);
  } catch (err) {
    showToast(`Error: ${err}`);
  }
}

if (btnApplyWhite) {
  btnApplyWhite.addEventListener("click", () => applyWhiteToHardware());
}

if (btnSaveWhite) {
  btnSaveWhite.addEventListener("click", () => {
    const data = {
      ...whiteState,
      hex: rgbToHex(whiteState.r, whiteState.g, whiteState.b),
      savedAt: new Date().toISOString(),
    };
    localStorage.setItem(WHITE_CALIB_STORAGE_KEY, JSON.stringify(data));
    showToast(`Calibrated neutral white saved! (RGB: ${whiteState.r}, ${whiteState.g}, ${whiteState.b})`);
    if (btnLoadSavedWhite) btnLoadSavedWhite.style.display = "inline-flex";
  });
}

if (btnLoadSavedWhite) {
  btnLoadSavedWhite.addEventListener("click", async () => {
    try {
      const raw = localStorage.getItem(WHITE_CALIB_STORAGE_KEY);
      if (!raw) return;
      const data = JSON.parse(raw);
      whiteState.r = data.r;
      whiteState.g = data.g;
      whiteState.b = data.b;
      calibPresetBtns.forEach((b) => b.classList.remove("active"));
      syncWhiteCalibrationUI("all");
      await applyWhiteToHardware();
      showToast(`Loaded user calibrated white: RGB(${data.r}, ${data.g}, ${data.b})`);
    } catch (e) {
      console.warn("Could not parse saved white:", e);
    }
  });
}

function checkSavedWhiteProfile() {
  const raw = localStorage.getItem(WHITE_CALIB_STORAGE_KEY);
  if (raw && btnLoadSavedWhite) {
    btnLoadSavedWhite.style.display = "inline-flex";
  }
}

// --------------------------------------------------------------------------
// Hardware Status Check on Mount
// --------------------------------------------------------------------------
async function checkHardware() {
  const mbInd = document.getElementById("mb-indicator");
  const gpuInd = document.getElementById("gpu-indicator");

  if (mbInd) {
    try {
      await invokeCommand("get_mb_status");
      mbInd.className = "telemetry-pill active";
      const title = mbInd.querySelector(".pill-title");
      if (title) title.innerText = "B550 GAMING PLUS (Ready)";
    } catch (e) {
      mbInd.className = "telemetry-pill error";
      const title = mbInd.querySelector(".pill-title");
      if (title) title.innerText = "B550 Offline";
    }
  }

  if (gpuInd) {
    try {
      const gpu = await invokeCommand("get_gpu_status");
      gpuInd.className = "telemetry-pill active";
      const title = gpuInd.querySelector(".pill-title");
      if (title) title.innerText = `RTX 3060 Ti (${gpu.address})`;
    } catch (e) {
      gpuInd.className = "telemetry-pill";
      const title = gpuInd.querySelector(".pill-title");
      if (title) title.innerText = "GPU Standby";
    }
  }
}

// --------------------------------------------------------------------------
// Initialization
// --------------------------------------------------------------------------
window.addEventListener("DOMContentLoaded", () => {
  if (viewTabsController) viewTabsController.init();
  if (effectTabsController) effectTabsController.init();

  syncStudioControls("all");
  recalculateTestSpectrum();
  renderSavedCheckpointBox();
  syncWhiteCalibrationUI("all");
  checkSavedWhiteProfile();
  checkHardware();
});

window.addEventListener("resize", () => {
  if (viewTabsController) viewTabsController.init();
  if (effectTabsController) effectTabsController.init();
});
