fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("logo.ico");
        res.compile().unwrap();
    }

    #[cfg(target_os = "macos")]
    {
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let profile = std::env::var("PROFILE").unwrap();
        let mut target_dir = std::path::PathBuf::from(out_dir);
        while target_dir.file_name().map(|n| n.to_str().unwrap()) != Some("target") {
            if !target_dir.pop() {
                break;
            }
        }
        let dest_dir = target_dir.join(profile);
        let dest_path = dest_dir.join("tray_helper");

        println!("cargo:rerun-if-changed=tray_helper.swift");

        let status = std::process::Command::new("swiftc")
            .arg("tray_helper.swift")
            .arg("-o")
            .arg(&dest_path)
            .status()
            .expect("Failed to compile tray_helper.swift using swiftc");

        if !status.success() {
            panic!("swiftc failed to compile tray_helper.swift");
        }
    }
}
