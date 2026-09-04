// ==========================================================================
// FanRGB Controller - Single Page 20-Color & Effect Orchestration
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
const tabsPill = tabsBar.querySelector(".t-tabs-pill");
const tabButtons = [...tabsBar.querySelectorAll(".t-tab")];

function movePillTo(tab, animate = true) {
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

    const mode = tab.getAttribute("data-mode");
    currentMode = mode;

    document.getElementById("active-effect-pill").innerText = `MODE: ${mode.toUpperCase()}`;
    await sendHardwareUpdate();
  });
});

// --------------------------------------------------------------------------
// Color Utilities & 20-Color Selection
// --------------------------------------------------------------------------
function hexToRgb(hex) {
  const clean = hex.replace("#", "");
  const num = parseInt(clean, 16);
  return {
    r: (num >> 16) & 255,
    g: (num >> 8) & 255,
    b: num & 255,
  };
}

// Wire 20 color buttons
const colorButtons = document.querySelectorAll(".color-card");
colorButtons.forEach((btn) => {
  btn.addEventListener("click", async () => {
    colorButtons.forEach((b) => b.classList.remove("active"));
    btn.classList.add("active");

    const hex = btn.getAttribute("data-hex");
    const name = btn.getAttribute("data-name");
    const rgb = hexToRgb(hex);

    currentColor = { r: rgb.r, g: rgb.g, b: rgb.b, hex, name };

    document.getElementById("custom-picker").value = hex;
    document.getElementById("custom-hex").value = hex.toUpperCase();

    await sendHardwareUpdate();
  });
});

// Custom Color Picker and Hex Input
const customPicker = document.getElementById("custom-picker");
const customHex = document.getElementById("custom-hex");

customPicker.addEventListener("input", async (e) => {
  const hex = e.target.value;
  customHex.value = hex.toUpperCase();
  const rgb = hexToRgb(hex);

  currentColor = { r: rgb.r, g: rgb.g, b: rgb.b, hex, name: `Custom (${hex.toUpperCase()})` };
  colorButtons.forEach((b) => b.classList.remove("active"));

  await sendHardwareUpdate();
});

customHex.addEventListener("change", async (e) => {
  let hex = e.target.value;
  if (!hex.startsWith("#")) hex = `#${hex}`;
  if (/^#[0-9A-Fa-f]{6}$/.test(hex)) {
    customPicker.value = hex;
    const rgb = hexToRgb(hex);
    currentColor = { r: rgb.r, g: rgb.g, b: rgb.b, hex, name: `Custom (${hex.toUpperCase()})` };
    colorButtons.forEach((b) => b.classList.remove("active"));
    await sendHardwareUpdate();
  }
});

// --------------------------------------------------------------------------
// Hardware Dispatch Engine
// --------------------------------------------------------------------------
async function sendHardwareUpdate() {
  const isOff = currentMode === "off";
  const r = isOff ? 0 : currentColor.r;
  const g = isOff ? 0 : currentColor.g;
  const b = isOff ? 0 : currentColor.b;

  // Update UI preview
  const dot = document.getElementById("active-preview-dot");
  dot.style.background = isOff ? "#18181b" : currentColor.hex;
  dot.style.boxShadow = isOff ? "none" : `0 0 16px ${currentColor.hex}`;

  const stateStr = isOff ? "System Lighting Powered Off" : `${currentColor.name} · ${currentMode.toUpperCase()}`;
  const shimmer = document.getElementById("active-state-text");
  shimmer.setAttribute("data-text", stateStr);
  shimmer.innerText = stateStr;

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

  try {
    await invokeCommand("get_mb_status");
    mbInd.className = "status-indicator active";
    mbInd.querySelector(".status-label").innerText = "B550 HID Active";
  } catch (e) {
    mbInd.className = "status-indicator error";
    mbInd.querySelector(".status-label").innerText = "B550 Offline";
  }

  try {
    const gpu = await invokeCommand("get_gpu_status");
    gpuInd.className = "status-indicator active";
    gpuInd.querySelector(".status-label").innerText = `RTX 3060 Ti (${gpu.address})`;
  } catch (e) {
    gpuInd.className = "status-indicator";
    gpuInd.querySelector(".status-label").innerText = "GPU Standby";
  }
}

window.addEventListener("DOMContentLoaded", () => {
  const activeTab = tabsBar.querySelector('.t-tab[aria-selected="true"]') || tabButtons[0];
  if (activeTab) movePillTo(activeTab, false);
  checkHardware();
});

window.addEventListener("resize", () => {
  const activeTab = tabsBar.querySelector('.t-tab[aria-selected="true"]');
  if (activeTab) movePillTo(activeTab, false);
});
