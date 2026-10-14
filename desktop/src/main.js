const { invoke } = window.__TAURI__.core;

let shownSignature = "";

function pct(v) {
  return Math.round(v * 100) + "%";
}

async function renderState(state) {
  // --- Source ---
  const sourceNameEl = document.querySelector("#source-name");
  const sourceVolumeEl = document.querySelector("#source-volume");
  const sourcePercentEl = document.querySelector("#source-percent");
  const sourceMuteEl = document.querySelector("#source-mute");
  
  sourceNameEl.textContent = state.source_name;
  sourceVolumeEl.value = state.source_volume;
  sourcePercentEl.textContent = pct(state.source_volume);
  
  // assign (not addEventListener) so re-renders dont stack listeners
  sourceVolumeEl.oninput = (e) => {
    const v = Number(e.target.value);
    sourcePercentEl.textContent = pct(v);
    invoke("set_source_volume", { volume: v });
  }
  
  let sourceMuted = state.source_muted;
  sourceMuteEl.textContent = sourceMuted ? "🔇" : "🔊";
  sourceMuteEl.classList.toggle("muted", sourceMuted);

  sourceMuteEl.onclick = () => {
    sourceMuted = !sourceMuted;
    sourceMuteEl.textContent = sourceMuted ? "🔇" : "🔊";
    sourceMuteEl.classList.toggle("muted", sourceMuted);
    invoke("set_source_mute", { muted: sourceMuted });
  }
  
  // --- Sinks ---
  const sinksEl = document.querySelector("#sinks");
  sinksEl.innerHTML = ""; // Clear before (re)building

  state.sinks.forEach((sink, i) => {
    const cardEl = document.createElement("div");
    cardEl.className = "card";

    const topEl = document.createElement("div");
    topEl.className = "card-top";

    const iconEl = document.createElement("span");
    iconEl.className = "icon";
    iconEl.textContent = "🎧";

    const nameEl = document.createElement("span");
    nameEl.className = "device-name";
    nameEl.textContent = sink.name;

    topEl.appendChild(iconEl);
    topEl.appendChild(nameEl);

    const controlsEl = document.createElement("div");
    controlsEl.className = "controls";

    const sliderEl = document.createElement("input");
    sliderEl.type = "range";
    sliderEl.className = "volume-slider";
    sliderEl.min = 0;
    sliderEl.max = 1;
    sliderEl.step = 0.01;
    sliderEl.value = sink.volume;

    const percentEl = document.createElement("span");
    percentEl.className = "percent";
    percentEl.textContent = pct(sink.volume);

    sliderEl.oninput = (e) => {
      const v = Number(e.target.value);
      percentEl.textContent = pct(v);
      invoke("set_volume", {index: i, volume: v });
    };

    const muteEl = document.createElement("button");
    muteEl.className = "mute-btn";
    let muted = sink.muted;
    muteEl.textContent = muted ? "🔇" : "🔊";
    muteEl.classList.toggle("muted", muted);

    muteEl.onclick = () => {
      muted = !muted;
      muteEl.textContent = muted ? "🔇" : "🔊";
      muteEl.classList.toggle("muted", muted);
      invoke("set_sink_mute", { index: i, muted });
    }

    controlsEl.appendChild(sliderEl);
    controlsEl.appendChild(percentEl);
    controlsEl.appendChild(muteEl);

    cardEl.appendChild(topEl);
    cardEl.appendChild(controlsEl);
    
    sinksEl.appendChild(cardEl);
  });
}

async function refresh() {
  const state = await invoke("get_state");
  const signature = state.source_name + "|" + state.sinks.map((s) => s.name).join("||");
  if (signature !== shownSignature) {
    renderState(state);
    shownSignature = signature;
  }
}

refresh();
setInterval(refresh, 1500);




// let greetInputEl;
// let greetMsgEl;

// async function greet() {
//   // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
//   greetMsgEl.textContent = await invoke("greet", { name: greetInputEl.value });
// }

// window.addEventListener("DOMContentLoaded", () => {
//   greetInputEl = document.querySelector("#greet-input");
//   greetMsgEl = document.querySelector("#greet-msg");
//   document.querySelector("#greet-form").addEventListener("submit", (e) => {
//     e.preventDefault();
//     greet();
//   });
// });
