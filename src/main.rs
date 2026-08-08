mod app;
mod export;
mod models;
mod parser;

use eframe::egui;
use egui::IconData;
use std::fs;
use std::path::PathBuf;

const APP_NAME: &str = "clashpass";
const APP_DISPLAY_NAME: &str = "ClashPass";
const APP_COMMENT: &str = "Password Conflict Resolver";

fn icon_256() -> &'static [u8] {
    include_bytes!("../icons/clashpass_256.png")
}

fn load_icon() -> IconData {
    let decoder = png::Decoder::new(std::io::Cursor::new(icon_256()));
    let mut reader = decoder.read_info().expect("Failed to decode icon PNG");
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).expect("Failed to read icon frame");
    buf.truncate(info.buffer_size());
    IconData {
        rgba: buf,
        width: info.width,
        height: info.height,
    }
}

fn home_dir() -> PathBuf {
    dirs::home_dir().expect("Cannot determine home directory")
}

fn desktop_file_path() -> PathBuf {
    home_dir().join(".local/share/applications").join(format!("{APP_NAME}.desktop"))
}

fn icon_dir() -> PathBuf {
    home_dir().join(".local/share/icons/hicolor/256x256/apps")
}

fn local_bin() -> PathBuf {
    home_dir().join(".local/bin")
}

fn is_installed() -> bool {
    desktop_file_path().exists()
}

fn install() {
    let bin_dir = local_bin();
    let app_dir = icon_dir();
    let desk_dir = desktop_file_path().parent().unwrap().to_path_buf();

    fs::create_dir_all(&bin_dir).expect("Failed to create ~/.local/bin");
    fs::create_dir_all(&app_dir).expect("Failed to create icon directory");
    fs::create_dir_all(&desk_dir).expect("Failed to create applications directory");

    // Copy binary
    let current_exe = std::env::current_exe().expect("Cannot determine current executable path");
    let dest = bin_dir.join(APP_NAME);
    fs::copy(&current_exe, &dest).expect("Failed to copy binary to ~/.local/bin");
    fs::set_permissions(&dest, std::os::unix::fs::PermissionsExt::from_mode(0o755)).ok();

    // Write .desktop file
    let desktop = format!(
        r#"[Desktop Entry]
Name={name}
Comment={comment}
Exec={exec}
Icon={icon}
Terminal=false
Type=Application
Categories=Utility;Security;
Keywords=password;manager;conflict;csv;
StartupWMClass={name}
"#,
        name = APP_NAME,
        comment = APP_COMMENT,
        exec = dest.display(),
        icon = APP_NAME,
    );
    fs::write(desktop_file_path(), desktop).expect("Failed to write .desktop file");

    // Write icon
    fs::write(app_dir.join(format!("{APP_NAME}.png")), icon_256()).expect("Failed to write icon");

    // Update desktop database
    std::process::Command::new("update-desktop-database")
        .arg(desk_dir)
        .output()
        .ok();

    println!("Installed successfully:");
    println!("  Binary:  {}", dest.display());
    println!("  Desktop: {}", desktop_file_path().display());
    println!("  Icon:    {}", app_dir.join(format!("{APP_NAME}.png")).display());
    println!();
    println!("You can now launch {} from your app launcher.", APP_DISPLAY_NAME);
}

fn uninstall() {
    let _ = fs::remove_file(desktop_file_path());
    let _ = fs::remove_file(icon_dir().join(format!("{APP_NAME}.png")));
    let _ = fs::remove_file(local_bin().join(APP_NAME));

    let desk_dir = desktop_file_path().parent().unwrap().to_path_buf();
    std::process::Command::new("update-desktop-database")
        .arg(desk_dir)
        .output()
        .ok();

    println!("Uninstalled. Binary, desktop entry, and icon removed.");
}

fn launch_app() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("ClashPass — Password Conflict Resolver")
            .with_inner_size([1200.0, 700.0])
            .with_min_inner_size([800.0, 500.0])
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        APP_DISPLAY_NAME,
        options,
        Box::new(|cc| Ok(Box::new(app::PasswordComparerApp::new(cc)))),
    )
}

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().collect();

    if args.contains(&"--install".to_string()) {
        install();
        return Ok(());
    }

    if args.contains(&"--uninstall".to_string()) {
        uninstall();
        return Ok(());
    }

    // Auto-install on first run (Linux only)
    #[cfg(target_os = "linux")]
    if !is_installed() {
        install();
    }

    launch_app()
}
