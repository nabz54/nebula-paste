%global crate_version 1.0.0-rc.1
%global debug_package %{nil}

Name:           nebula-paste
Version:        1.0.0~rc.1
Release:        1%{?dist}
Summary:        Local clipboard history applet for COSMIC
License:        MPL-2.0 AND Apache-2.0 AND BSD-2-Clause AND MIT
URL:            https://github.com/nabz54/nebula-paste
Source0:        %{name}-%{crate_version}.tar.gz
Source1:        %{name}-cargo-vendor.tar.gz
BuildRequires:  cargo >= 1.93
BuildRequires:  rust >= 1.93
BuildRequires:  gcc-c++
BuildRequires:  cmake
BuildRequires:  make
BuildRequires:  pkgconfig(xkbcommon)
BuildRequires:  pkgconfig(wayland-client)
BuildRequires:  pkgconfig(fontconfig)
BuildRequires:  pkgconfig(freetype2)
BuildRequires:  desktop-file-utils
Recommends:     wtype

%description
A Rust/libcosmic clipboard applet with local history, collections and optional
image-text search. French/English OCR libraries and models are bundled.
Add Nebula Paste to the COSMIC panel after installation.
This is an upstream release candidate package, not an official Fedora repository package.

%prep
%setup -q -n %{name}-%{crate_version}
tar -xzf %{SOURCE1}

%build
export CARGO_TARGET_DIR=target
export CARGO_PROFILE_RELEASE_LTO=false
cargo build --release --locked --offline -j 2

%install
install -Dm755 target/release/nebula-paste %{buildroot}%{_bindir}/nebula-paste
for desktop in resources/*.desktop; do
    install -Dm644 "$desktop" "%{buildroot}%{_datadir}/applications/$(basename "$desktop")"
done
for icon in resources/*.svg; do
    install -Dm644 "$icon" "%{buildroot}%{_datadir}/icons/hicolor/scalable/apps/$(basename "$icon")"
done

%check
for desktop in resources/*.desktop; do desktop-file-validate "$desktop"; done
bash scripts/check-embedded-ocr.sh target/release/nebula-paste

%files
%license LICENSE vendor/licenses cargo-licenses
%doc README.md README.en.md CHANGELOG.md CHANGELOG.fr.md
%{_bindir}/nebula-paste
%{_datadir}/applications/io.github.nebulapaste.NebulaPaste.desktop
%{_datadir}/applications/io.github.nebulapaste.NebulaPaste.History.desktop
%{_datadir}/icons/hicolor/scalable/apps/io.github.nebulapaste.NebulaPaste.svg
%{_datadir}/icons/hicolor/scalable/apps/io.github.nebulapaste.NebulaPaste-symbolic.svg

%changelog
* Thu Sep 24 2026 Nebula Paste contributors - 1.0.0~rc.1-1
- Native popup blur, distinct release candidate and installation documentation

* Sun Sep 20 2026 Nebula Paste contributors - 0.9.0~beta.1-1
- History backup/restore, diagnostics and COSMIC reliability

* Mon Sep 14 2026 Nebula Paste contributors - 0.8.0~beta.1-1
- Reusable templates, literal fields and transactional JSON import/export.

* Sat Sep 12 2026 Nebula Paste contributors - 0.7.0~beta.1-1
- Initial upstream Fedora package with vendored Rust dependencies and bundled OCR.
