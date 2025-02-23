import { listen } from '@tauri-apps/api/event';
const { invoke } = window.__TAURI__.core;


window.addEventListener("DOMContentLoaded", async () => {
    const logContents = await invoke('get_log_contents');
    //document.getElementById('log').textContent = logContents;
    //console.log(logContents);
    
    const container = document.getElementById("jsoneditor");
    const editor = new JSONEditor(container, { mode: "tree" });
    editor.set(logContents);
    invoke("get_logger_path", {}).then((res) => {
        console.log('Logger Path:', res);
        document.getElementById("log-file-path").textContent = res
    });
});
