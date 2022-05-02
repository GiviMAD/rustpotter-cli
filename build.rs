use std::{env, fs, path::PathBuf};

fn main() {
    let base_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let mut out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    out_dir.pop();
    out_dir.pop();
    prepare_pv_recorder_lib(base_dir, out_dir);
}
fn prepare_pv_recorder_lib (base_dir: PathBuf, build_dir: PathBuf) {
    let pv_recorder_lib_path = fs::read_dir(build_dir)
        .unwrap()
        .find_map(|entry| {
            let mut path = entry.unwrap().path();
            if path.to_str().unwrap().contains(&"pv_recorder") {
                path.push("out");
                path.push("lib");
                if path.exists() {
                    return Some(path);
                }
            }
            None
        })
        .expect("Unable to find pv_recorder lib path");
    let mut dist_folder = base_dir.clone();
    dist_folder.push("dist");
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        let windows_folder = pv_recorder_lib_path.clone().join("windows");
        let windows_x86_64_binary = windows_folder.clone().join("amd64/libpv_recorder.dll");
        let windows_x86_64_binary_dest = dist_folder.clone().join("windows/amd64");
        fs::create_dir_all(windows_x86_64_binary_dest.clone()).unwrap();
        fs::copy(windows_x86_64_binary, windows_x86_64_binary_dest.join("libpv_recorder.dll")).unwrap();
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        let linux_folder = pv_recorder_lib_path.clone().join("linux");
        let linux_x86_64_binary = linux_folder.clone().join("x86_64/libpv_recorder.so");
        let linux_x86_64_binary_dest = dist_folder.clone().join("linux/x86_64");
        fs::create_dir_all(linux_x86_64_binary_dest.clone()).unwrap();
        fs::copy(linux_x86_64_binary, linux_x86_64_binary_dest.join("libpv_recorder.so")).unwrap();
    }
    #[cfg(all(target_os = "linux", any(target_arch = "arm",  target_arch = "aarch64")))]
    let rpi_folder = pv_recorder_lib_path.clone().join("raspberry-pi");
    #[cfg(all(target_os = "linux", target_arch = "arm"))]
    {
        let arm11_binary = rpi_folder.clone().join("arm11/libpv_recorder.so");
        let arm11_binary_dest = dist_folder.clone().join("raspberry-pi/arm11");
        fs::create_dir_all(arm11_binary_dest.clone()).unwrap();
        fs::copy(arm11_binary, arm11_binary_dest.join("libpv_recorder.so")).unwrap();

        let cortex_a7_binary = rpi_folder.clone().join("cortex-a7/libpv_recorder.so");
        let cortex_a7_binary_dest = dist_folder.clone().join("raspberry-pi/cortex-a7");
        fs::create_dir_all(cortex_a7_binary_dest.clone()).unwrap();
        fs::copy(
            cortex_a7_binary,
            cortex_a7_binary_dest.join("libpv_recorder.so"),
        )
        .unwrap();

        let cortex_a53_binary = rpi_folder.clone().join("cortex-a53/libpv_recorder.so");
        let cortex_a53_binary_dest = dist_folder.clone().join("raspberry-pi/cortex-a53");
        fs::create_dir_all(cortex_a53_binary_dest.clone()).unwrap();
        fs::copy(
            cortex_a53_binary,
            cortex_a53_binary_dest.join("libpv_recorder.so"),
        )
        .unwrap();

        let cortex_a72_binary = rpi_folder.clone().join("cortex-a72/libpv_recorder.so");
        let cortex_a72_binary_dest = dist_folder.clone().join("raspberry-pi/cortex-a72");
        fs::create_dir_all(cortex_a72_binary_dest.clone()).unwrap();
        fs::copy(
            cortex_a72_binary,
            cortex_a72_binary_dest.join("libpv_recorder.so"),
        )
        .unwrap();
    }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        let cortex_a53_aarch64_binary = rpi_folder
            .clone()
            .join("cortex-a53-aarch64/libpv_recorder.so");
        let cortex_a53_aarch64_binary_dest =
            dist_folder.clone().join("raspberry-pi/cortex-a53-aarch64");
        fs::create_dir_all(cortex_a53_aarch64_binary_dest.clone()).unwrap();
        fs::copy(
            cortex_a53_aarch64_binary,
            cortex_a53_aarch64_binary_dest.join("libpv_recorder.so"),
        )
        .unwrap();

        let cortex_a72_aarch64_binary = rpi_folder
            .clone()
            .join("cortex-a72-aarch64/libpv_recorder.so");
        let cortex_a72_aarch64_binary_dest =
            dist_folder.clone().join("raspberry-pi/cortex-a72-aarch64");
        fs::create_dir_all(cortex_a72_aarch64_binary_dest.clone()).unwrap();
        fs::copy(
            cortex_a72_aarch64_binary,
            cortex_a72_aarch64_binary_dest.join("libpv_recorder.so"),
        )
        .unwrap();
    }
    #[cfg(target_os = "macos")]
    let mac_folder = pv_recorder_lib_path.clone().join("mac");
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        let mac_x86_64_binary = mac_folder.clone().join("x86_64/libpv_recorder.dylib");
        let mac_x86_64_binary_dest = dist_folder.clone().join("mac/x86_64");
        fs::create_dir_all(mac_x86_64_binary_dest.clone()).unwrap();
        fs::copy(
            mac_x86_64_binary,
            mac_x86_64_binary_dest.join("libpv_recorder.dylib"),
        )
        .unwrap();
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        let mac_aarch64_binary = mac_folder.clone().join("arm64/libpv_recorder.dylib");
        let mac_aarch64_binary_dest = dist_folder.clone().join("mac/arm64");
        fs::create_dir_all(mac_aarch64_binary_dest.clone()).unwrap();
        fs::copy(
            mac_aarch64_binary,
            mac_aarch64_binary_dest.join("libpv_recorder.dylib"),
        )
        .unwrap();
    }
    println!("cargo:rerun-if-changed={:?}", dist_folder);
    println!("cargo:rerun-if-changed={:?}", pv_recorder_lib_path);
}