// ==========================================================================
// FanRGB Controller - Single Page 20-Color, Precision Sliders & Effects
// ==========================================================================

// Global state
let currentColor = { r: 255, g: 0, b: 0, hex: "#ff0000", name: "Pure Red" };
let currentMode = "static"; // "static", "breathing", "meteor", "flashing", "off"

// Robust IPC Bridge
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
    const errorMsg = "Tauri IPC bridge is not ready. Please launch through the desktop app.";
    console.error(errorMsg, cmd, args);
    showToast(`Error: ${errorMsg}`);
    throw new Error(errorMsg);
  }

  try {
    const result = await fn(cmd, args);
    return result;
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
// Transitions.dev: Sliding Tabs (Effect Mode Selector)
// --------------------------------------------------------------------------
const tabsBar = document.querySelector(".t-tabs");
const tabsPill = tabsBar?.querySelector(".t-tabs-pill");
const tabButtons = tabsBar ? [...tabsBar.querySelectorAll(".t-tab")] : [];

function movePillTo(tab, animate = true) {
  if (!tabsPill || !tab) return;
  if (!animate) {
    tabsPill.style.transition = "none";
  }
  tabsPill.style.transform = `translateX(${tab.offsetLeft}px)`;
  tabsPill.style.width = `${tab.offsetWidth}px`;
  if (!animate) {
    tabsPill.offsetHeight; // force reflow
    tabsPill.style.transition = "";
  }
}

tabButtons.forEach((tab) => {
  tab.addEventListener("click", async () => {
    tabButtons.forEach((t) => t.setAttribute("aria-selected", "false"));
    tab.setAttribute("aria-selected", "true");
    movePillTo(tab, true);

    const mode = tab.getAttribute("data-mode") || "static";
    currentMode = mode;

    const effectTag = document.getElementById("active-effect-tag");
    if (effectTag) {
      effectTag.innerText = mode.toUpperCase();
    }

    await sendHardwareUpdate();
  });
});

// --------------------------------------------------------------------------
// Color Utilities & State Synchronization
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

// Elements
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

function syncControls(source = "all") {
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

  if (source !== "picker" && customPicker) {
    customPicker.value = hex;
  }

  if (source !== "hex" && customHex) {
    customHex.value = hex.toUpperCase();
  }

  if (rgbSummary) {
    rgbSummary.innerText = `RGB: ${r}, ${g}, ${b}`;
  }

  updateOrbAndTelemetry();
}

function updateOrbAndTelemetry() {
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

// --------------------------------------------------------------------------
// Precision RGB Channel Sliders & Numbers
// --------------------------------------------------------------------------
let sliderThrottleTimer = null;

function handleSliderChange(source) {
  const r = parseInt(sliderR.value, 10) || 0;
  const g = parseInt(sliderG.value, 10) || 0;
  const b = parseInt(sliderB.value, 10) || 0;
  const hex = rgbToHex(r, g, b);

  currentColor = { r, g, b, hex, name: "Custom" };
  colorButtons.forEach((btn) => btn.classList.remove("active"));

  syncControls(source);

  // Throttle hardware updates during sliding to preserve USB bus bandwidth
  if (sliderThrottleTimer) clearTimeout(sliderThrottleTimer);
  sliderThrottleTimer = setTimeout(() => {
    sendHardwareUpdate();
  }, 40);
}

[sliderR, sliderG, sliderB].forEach((slider) => {
  if (!slider) return;
  slider.addEventListener("input", () => handleSliderChange("sliders"));
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

    handleSliderChange("numbers");
  });
});

// --------------------------------------------------------------------------
// 20-Color Studio Preset Cards
// --------------------------------------------------------------------------
colorButtons.forEach((btn) => {
  btn.addEventListener("click", async () => {
    colorButtons.forEach((b) => b.classList.remove("active"));
    btn.classList.add("active");

    const hex = btn.getAttribute("data-hex") || "#ff0000";
    const name = btn.getAttribute("data-name") || "Custom";
    const rgb = hexToRgb(hex);

    currentColor = { r: rgb.r, g: rgb.g, b: rgb.b, hex, name };
    syncControls("preset");
    await sendHardwareUpdate();
  });
});

// --------------------------------------------------------------------------
// Custom Native Color Picker & Hex Input
// --------------------------------------------------------------------------
if (customPicker) {
  customPicker.addEventListener("input", async (e) => {
    const hex = e.target.value;
    const rgb = hexToRgb(hex);

    currentColor = { r: rgb.r, g: rgb.g, b: rgb.b, hex, name: `Custom (${hex.toUpperCase()})` };
    colorButtons.forEach((b) => b.classList.remove("active"));
    syncControls("picker");

    if (sliderThrottleTimer) clearTimeout(sliderThrottleTimer);
    sliderThrottleTimer = setTimeout(() => {
      sendHardwareUpdate();
    }, 40);
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
      syncControls("hex");
      await sendHardwareUpdate();
    }
  });
}

// --------------------------------------------------------------------------
// Hardware Dispatch Engine
// --------------------------------------------------------------------------
async function sendHardwareUpdate() {
  const isOff = currentMode === "off";
  const r = isOff ? 0 : currentColor.r;
  const g = isOff ? 0 : currentColor.g;
  const b = isOff ? 0 : currentColor.b;

  updateOrbAndTelemetry();

  const stateStr = isOff
    ? "System Lighting Powered Off"
    : `${currentColor.name} · ${currentMode.toUpperCase()} (${r}, ${g}, ${b})`;

  showToast(`Applying ${stateStr}...`);

  try {
    await invokeCommand("apply_lighting", {
      r,
      g,
      b,
      mode: currentMode,
    });
    showToast(`Active: ${stateStr}`);
  } catch (err) {
    showToast(`Error: ${err}`);
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

window.addEventListener("DOMContentLoaded", () => {
  const activeTab = tabsBar?.querySelector('.t-tab[aria-selected="true"]') || tabButtons[0];
  if (activeTab) movePillTo(activeTab, false);
  syncControls("all");
  checkHardware();
});

window.addEventListener("resize", () => {
  const activeTab = tabsBar?.querySelector('.t-tab[aria-selected="true"]');
  if (activeTab) movePillTo(activeTab, false);
});
