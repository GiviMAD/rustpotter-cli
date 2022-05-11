use include_dir::{include_dir, Dir};
use std::io::Write;
use std::path::PathBuf;
use tempfile::{Builder as TempFileBuilder, TempPath};

static _DIST_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/dist");

pub fn _get_pv_recorder_lib() -> TempPath {
    let rel_path = _base_library_path();
    let native_lib = _DIST_DIR.get_file(rel_path).unwrap();
    let mut file = TempFileBuilder::new()
        .prefix("pv_recorder")
        .suffix(".tmp")
        .rand_bytes(8)
        .tempfile()
        .unwrap();
    file.write_all(native_lib.contents())
        .expect("Unable to write to temporal file");
    file.flush().expect("Unable to write to temporal file");
    file.into_temp_path()
}

#[cfg(all(target_os = "linux", any(target_arch = "arm", target_arch = "aarch64")))]
fn find_machine_type() -> String {
    use std::process::Command;

    let cpu_info = Command::new("cat")
        .arg("/proc/cpuinfo")
        .output()
        .expect("Failed to retrieve cpu info");
    let cpu_part_list = std::str::from_utf8(&cpu_info.stdout)
        .unwrap()
        .split("\n")
        .filter(|x| x.contains("CPU part"))
        .collect::<Vec<_>>();

    if cpu_part_list.len() == 0 {
        panic!("Unsupported CPU");
    }

    let cpu_part = cpu_part_list[0]
        .split(" ")
        .collect::<Vec<_>>()
        .pop()
        .unwrap()
        .to_lowercase();

    let machine = match cpu_part.as_str() {
        "0xb76" => "arm11",
        "0xc07" => "cortex-a7",
        "0xd03" => "cortex-a53",
        "0xd07" => "cortex-a57",
        "0xd08" => "cortex-a72",
        "0xc08" => "beaglebone",
        _ => "unsupported",
    };

    String::from(machine)
}

#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
fn _base_library_path() -> PathBuf {
    PathBuf::from("mac/x86_64/libpv_recorder.dylib")
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn _base_library_path() -> PathBuf {
    PathBuf::from("mac/arm64/libpv_recorder.dylib")
}

#[cfg(target_os = "windows")]
fn _base_library_path() -> PathBuf {
    PathBuf::from("windows/amd64/libpv_recorder.dll")
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn _base_library_path() -> PathBuf {
    PathBuf::from("linux/x86_64/libpv_recorder.so")
}

#[cfg(all(target_os = "linux", any(target_arch = "arm", target_arch = "aarch64")))]
fn _base_library_path() -> PathBuf {
    const RPI_MACHINES: [&str; 4] = ["arm11", "cortex-a7", "cortex-a53", "cortex-a72"];

    let machine = find_machine_type();
    match machine.as_str() {
        machine if RPI_MACHINES.contains(&machine) => {
            if cfg!(target_arch = "aarch64") {
                PathBuf::from(format!(
                    "raspberry-pi/{}-aarch64/libpv_recorder.so",
                    &machine
                ))
            } else {
                PathBuf::from(format!("raspberry-pi/{}/libpv_recorder.so", &machine))
            }
        }
        _ => {
            eprintln!("WARNING: Falling back to the armv6-based (Raspberry Pi Zero) library. This is not tested nor optimal.\nFor the model, use Raspberry Pi's models");
            PathBuf::from("raspberry-pi/arm11/libpv_recorder.so")
        }
    }
}
