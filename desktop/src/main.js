const { invoke } = window.__TAURI__.core;

async function render() {
  const state = await invoke("get_state");

  // --- Source ---
  const sourceNameEl = document.querySelector("#source-name");
  const sourceVolumeEl = document.querySelector("#source-volume");
  
  sourceNameEl.textContent = state.source_name;
  sourceVolumeEl.textContent = state.source_volume;
  
  sourceVolumeEl.addEventListener("input", (e) => {
    invoke("set_source_volume", {volume: Number(e.target.value) });
  });
  
  // --- Sinks ---
  const sinksEl = document.querySelector("#sinks");
  sinksEl.innerHTML = ""; // Clear before (re)building

  state.sinks.forEach((sink, i) => {
    const row = document.createElement("div");

    const label = document.createElement("span");
    label.textContent = sink.name;

    const slider = document.createElement("input");
    slider.type = "range";
    slider.min = 0;
    slider.max = 1;
    slider.step = 0.01;
    slider.value = sink.volume;

    slider.addEventListener("input", (e) => {
      invoke("set_volume", {index: i, volume: Number(e.target.value) });
    });

    row.append(label);
    row.appendChild(slider);
    sinksEl.appendChild(row);
  });
}

render();




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
