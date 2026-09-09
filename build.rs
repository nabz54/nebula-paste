//! Build the bundled OCR libraries. This script never downloads source code.
use std::{
    env, fs,
    io,
    path::{Path, PathBuf},
    process::Command,
};
fn source_archive(vendor: &Path, out: &Path, package: &str) -> PathBuf {
    if package != "leptonica-1.85.0" {
        return vendor.join(format!("{package}.tar.gz"));
    }
    let archive = out.join(format!("{package}.tar.gz"));
    let mut output = fs::File::create(&archive).expect("Créer l’archive OCR temporaire");
    for part in ["part1", "part2"] {
        let mut input = fs::File::open(vendor.join(format!("{package}.tar.gz.{part}")))
            .expect("Ouvrir une partie de l’archive OCR");
        io::copy(&mut input, &mut output).expect("Reconstituer l’archive OCR");
    }
    archive
}
fn run(command: &mut Command) {
    let result = command.status().unwrap_or_else(|e| {
        panic!("Impossible de lancer {command:?}: {e}. Installe cmake, make, gcc et g++.")
    });
    assert!(
        result.success(),
        "Échec de la compilation OCR : {command:?}"
    );
}
fn configure(source: &Path, build: &Path, options: &[&str]) -> Command {
    let mut cmd = Command::new("cmake");
    cmd.arg("-S")
        .arg(source)
        .arg("-B")
        .arg(build)
        .args([
            "-G",
            "Unix Makefiles",
            "-DCMAKE_BUILD_TYPE=Release",
            "-DBUILD_SHARED_LIBS=OFF",
            "-DCMAKE_POSITION_INDEPENDENT_CODE=ON",
            "-DSW_BUILD=OFF",
        ])
        .args(options);
    cmd
}
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=vendor");
    assert_eq!(
        env::var("HOST").unwrap(),
        env::var("TARGET").unwrap(),
        "Cette distribution prend en charge la compilation native seulement."
    );
    assert_eq!(env::var("CARGO_CFG_TARGET_OS").unwrap(), "linux");
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let vendor = root.join("vendor");
    run(Command::new("sha256sum")
        .args(["--check", "--status", "SHA256SUMS"])
        .current_dir(&vendor));
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let sources = out.join("ocr-sources");
    let manifest = fs::read(vendor.join("SHA256SUMS")).unwrap();
    let stamp = sources.join("verified-manifest");
    if fs::read(&stamp).ok().as_deref() != Some(manifest.as_slice()) {
        if sources.exists() {
            fs::remove_dir_all(&sources).unwrap();
        }
        fs::create_dir_all(&sources).unwrap();
        for package in ["leptonica-1.85.0", "tesseract-5.5.1"] {
            run(Command::new("tar")
                .arg("-xzf")
                .arg(source_archive(&vendor, &out, package))
                .arg("-C")
                .arg(&sources));
        }
        fs::write(stamp, manifest).unwrap();
    }
    let jobs = env::var("NUM_JOBS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(2)
        .clamp(1, 4)
        .to_string();
    let prefix = out.join("ocr-install");
    let lept = out.join("leptonica-build");
    run(configure(
        &sources.join("leptonica-1.85.0"),
        &lept,
        &[
            "-DBUILD_PROG=OFF",
            "-DENABLE_ZLIB=OFF",
            "-DENABLE_PNG=OFF",
            "-DENABLE_JPEG=OFF",
            "-DENABLE_GIF=OFF",
            "-DENABLE_TIFF=OFF",
            "-DENABLE_WEBP=OFF",
            "-DENABLE_OPENJPEG=OFF",
        ],
    )
    .arg(format!("-DCMAKE_INSTALL_PREFIX={}", prefix.display())));
    run(Command::new("cmake")
        .arg("--build")
        .arg(&lept)
        .args(["--parallel", &jobs]));
    run(Command::new("cmake").arg("--install").arg(&lept));
    let tess = out.join("tesseract-build");
    run(configure(
        &sources.join("tesseract-5.5.1"),
        &tess,
        &[
            "-DCMAKE_CXX_FLAGS=-DTESSERACT_DISABLE_DEBUG_FONTS -DTESSERACT_IMAGEDATA_AS_PIX",
            "-DBUILD_TRAINING_TOOLS=OFF",
            "-DBUILD_TESTS=OFF",
            "-DOPENMP_BUILD=OFF",
            "-DGRAPHICS_DISABLED=ON",
            "-DDISABLED_LEGACY_ENGINE=ON",
            "-DDISABLE_TIFF=ON",
            "-DDISABLE_ARCHIVE=ON",
            "-DDISABLE_CURL=ON",
            "-DENABLE_NATIVE=OFF",
        ],
    )
    .arg(format!("-DCMAKE_PREFIX_PATH={}", prefix.display())));
    run(Command::new("cmake").arg("--build").arg(&tess).args([
        "--target",
        "libtesseract",
        "--parallel",
        &jobs,
    ]));
    println!("cargo:rustc-link-search=native={}", tess.display());
    println!(
        "cargo:rustc-link-search=native={}",
        prefix.join("lib").display()
    );
    println!("cargo:rustc-link-lib=static=tesseract");
    println!("cargo:rustc-link-lib=static=leptonica");
    println!("cargo:rustc-link-lib=dylib=stdc++");
    println!("cargo:rustc-link-lib=dylib=m");
    println!("cargo:rustc-link-lib=dylib=pthread");
}
