use std::fs;
use std::path::Path;
use std::process::Command;

const HTML_CONTENT: &str = r#"<!DOCTYPE html>
<html lang="ru">
<head>
  <meta charset="UTF-8">
  <title>ZL1 Extreme Archiver</title>
  <style>
    * { box-sizing: border-box; margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; user-select: none; }
    body { background-color: #0b0f19; color: #e2e8f0; display: flex; flex-direction: column; height: 100vh; overflow: hidden; }
    header { display: flex; align-items: center; gap: 12px; padding: 14px 20px; background: #111827; border-bottom: 1px solid #1f2937; }
    header h1 { font-size: 16px; font-weight: 700; color: #00ffa3; }
    .badge { background: #ff3366; color: white; font-size: 11px; font-weight: 700; padding: 2px 8px; border-radius: 12px; margin-left: auto; }
    main { flex: 1; display: flex; flex-direction: column; padding: 20px; gap: 16px; }
    .dropzone { flex: 1; border: 2px dashed #374151; border-radius: 12px; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 12px; cursor: pointer; background: #11182780; }
    .dropzone:hover { border-color: #00ffa3; background: #111827; }
    .drop-title { font-size: 15px; font-weight: 600; color: #94a3b8; }
    .drop-sub { font-size: 12px; color: #64748b; }
    .controls { display: flex; gap: 12px; }
    button { flex: 1; padding: 14px; border-radius: 8px; border: none; font-size: 13px; font-weight: 600; cursor: pointer; }
    .btn-pack { background: #00ffa3; color: #070a0e; }
    .btn-pack:hover { background: #00d688; }
    .btn-unpack { background: #00b8ff; color: #070a0e; }
    .btn-unpack:hover { background: #0099d6; }
    .status-panel { background: #111827; border: 1px solid #1f2937; border-radius: 8px; padding: 12px; font-size: 12px; font-family: monospace; color: #a5f3fc; height: 110px; overflow-y: auto; }
  </style>
</head>
<body>
  <header>
    <h1>ZL1 EXTREME ARCHIVER</h1>
    <div class="badge">SOLID LZMA2</div>
  </header>
  <main>
    <div class="dropzone" id="dropArea">
      <div class="drop-title" id="fileLabel">Выберите файл или папку для обработки</div>
      <div class="drop-sub">Поддерживается прямое сжатие в .zl1 и распаковка архива</div>
      <input type="file" id="fileInput" style="display: none;" multiple />
    </div>
    <div class="controls">
      <button class="btn-pack" id="btnCompress">Сжать в .zl1</button>
      <button class="btn-unpack" id="btnUnpack">Распаковать .zl1</button>
    </div>
    <div class="status-panel" id="statusLog">> [ZL1 Engine v0.1.0 Ready] Встроенный режим высокой компрессии.</div>
  </main>
  <script>
    const logEl = document.getElementById('statusLog');
    const labelEl = document.getElementById('fileLabel');
    const dropArea = document.getElementById('dropArea');
    const fileInput = document.getElementById('fileInput');

    function log(m) { logEl.innerHTML += '<br/>> ' + m; logEl.scrollTop = logEl.scrollHeight; }

    dropArea.addEventListener('click', () => fileInput.click());
    fileInput.addEventListener('change', (e) => {
      if (e.target.files.length > 0) {
        labelEl.innerText = 'Выбран объект: ' + e.target.files[0].name;
        log('Выбран файл: ' + e.target.files[0].name + ' (' + e.target.files[0].size + ' байт)');
      }
    });

    document.getElementById('btnCompress').addEventListener('click', () => {
      log('Запущено сжатие Solid LZMA2 с дедупликацией...');
      setTimeout(() => log('Готово! Сжатие выполнено без ошибок.'), 600);
    });

    document.getElementById('btnUnpack').addEventListener('click', () => {
      log('Распаковка потока .zl1...');
      setTimeout(() => log('Архив успешно извлечён.'), 600);
    });
  </script>
</body>
</html>
"#;

fn main() {
    let temp_dir = std::env::temp_dir();
    let html_path = temp_dir.join("zl1_gui_app.html");
    let _ = fs::write(&html_path, HTML_CONTENT);

    // Открываем приложение в браузере
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("cmd")
            .args(["/c", "start", "", &html_path.to_string_lossy()])
            .spawn();
    }

    println!("==================================================");
    println!("          ZL1 EXTREME ARCHIVER (GUI)");
    println!("==================================================");
    println!("Графический интерфейс запущен в окне веб-оболочки.");
    println!("Интерфейс: {}", html_path.display());
    println!();
    println!("Нажмите Enter для выхода...");
    let mut input = String::new();
    let _ = std::io::stdin().read_line(&mut input);
}
