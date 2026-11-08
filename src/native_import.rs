use std::process::Command;
use std::sync::Mutex;

static PICKED: Mutex<Vec<Vec<u8>>> = Mutex::new(Vec::new());
static NOTICE: Mutex<Option<String>> = Mutex::new(None);

pub fn take() -> Vec<Vec<u8>> {
    PICKED.lock().map(|mut p| std::mem::take(&mut *p)).unwrap_or_default()
}

pub fn take_notice() -> Option<String> {
    NOTICE.lock().ok().and_then(|mut n| n.take())
}

fn notice(text: String) {
    if let Ok(mut n) = NOTICE.lock() {
        *n = Some(text);
    }
}

fn dialog_command() -> Vec<Command> {
    let mut commands = Vec::new();
    if cfg!(target_os = "windows") {
        let mut c = Command::new("powershell");
        c.args(["-NoProfile", "-Command", "Add-Type -AssemblyName System.Windows.Forms; $d = New-Object System.Windows.Forms.OpenFileDialog; $d.Filter = 'ZIP|*.zip'; $d.Multiselect = $true; if ($d.ShowDialog() -eq 'OK') { $d.FileNames }"]);
        commands.push(c);
    } else if cfg!(target_os = "macos") {
        let mut c = Command::new("osascript");
        c.args(["-e", "POSIX path of (choose file of type {\"public.zip-archive\"})"]);
        commands.push(c);
    } else {
        let mut zenity = Command::new("zenity");
        zenity.args(["--file-selection", "--multiple", "--separator=\n", "--file-filter=*.zip"]);
        commands.push(zenity);
        let mut kdialog = Command::new("kdialog");
        kdialog.args(["--getopenfilename", ".", "*.zip"]);
        commands.push(kdialog);
    }
    commands
}

fn run_dialog() -> Result<Vec<String>, String> {
    for mut command in dialog_command() {
        let Ok(output) = command.output() else { continue };
        let text = String::from_utf8_lossy(&output.stdout).into_owned();
        return Ok(text.lines().map(str::trim).filter(|l| !l.is_empty()).map(String::from).collect());
    }
    Err("no-dialog".into())
}

fn load(paths: Vec<String>) {
    for path in paths {
        match std::fs::read(&path) {
            Ok(bytes) => {
                if let Ok(mut picked) = PICKED.lock() {
                    picked.push(bytes);
                }
            }
            Err(e) => notice(format!("{path}: {e}")),
        }
    }
}

pub fn pick() {
    let spawned = std::thread::Builder::new().name("import-picker".into()).spawn(|| match run_dialog() {
        Ok(paths) => load(paths),
        Err(_) => notice(crate::i18n::tr3(
            "Sem seletor de arquivos: instale zenity ou kdialog, ou coloque as pastas em chars/ e stages/.",
            "No file picker found: install zenity or kdialog, or place folders in chars/ and stages/.",
            "No hay selector de archivos: instala zenity o kdialog, o coloca las carpetas en chars/ y stages/.",
        ).to_string()),
    });
    if spawned.is_err() {
        notice("import-picker".into());
    }
}
