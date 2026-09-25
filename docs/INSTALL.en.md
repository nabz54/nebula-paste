# Installation and upgrades

[Français](INSTALL.md)

Package target: **Fedora 44 x86_64 running COSMIC**. Unsigned upstream package; other distributions are not claimed as tested. Version 1.0.0-rc.1 is a release candidate, not a stable release. Public source clones require no GitHub account.

## Download

Once published, the candidate RPM and `SHA256SUMS` are attached to [the release](https://github.com/nabz54/nebula-paste/releases/tag/v1.0.0-rc.1). Before publication, use `nebula-paste-fedora44-rpm` from a successful GitHub Actions build of `release/1.0-rc1`; artifact downloads may require GitHub authentication.

For a published release, no login is required:

```bash
package_dir=$(mktemp -d "$HOME/nebula-1.0-XXXXXX")
cd "$package_dir"
base='https://github.com/nabz54/nebula-paste/releases/download/v1.0.0-rc.1'
curl --fail --location --output nebula-paste-1.0.0~rc.1-1.fc44.x86_64.rpm "$base/nebula-paste-1.0.0.rc.1-1.fc44.x86_64.rpm" &&
curl --fail --location --output SHA256SUMS "$base/SHA256SUMS"
grep -F '  nebula-paste-1.0.0~rc.1-1.fc44.x86_64.rpm' SHA256SUMS | sha256sum -c -
```

Proceed only if verification succeeds. The source RPM also listed in the manifest is not required for installation.

## Remove conflicting local launchers

Remove the applet from the panel and close its history window. If installed using `scripts/install.sh`, run `bash scripts/uninstall.sh` from that project. History and preferences remain intact. Without the source checkout, back up only the launch files:

```bash
pkill -x nebula-paste || true
backup_dir=$(mktemp -d "$HOME/nebula-old-installation-XXXXXX")
applications="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
for file in \
  "$HOME/.local/bin/nebula-paste" \
  "$applications/io.github.nebulapaste.NebulaPaste.desktop" \
  "$applications/io.github.nebulapaste.NebulaPaste.History.desktop"
do
  if [ -f "$file" ]; then mv -- "$file" "$backup_dir/"; fi
done
hash -r
```

Do not delete the data/configuration directories. User launchers shadow the RPM launchers even when `/usr/bin/nebula-paste --version` reports the new version. Update shortcuts pointing to the old user binary to `/usr/bin/nebula-paste --toggle`.

## Install or upgrade

From the directory containing the verified RPM:

```bash
sudo dnf install './nebula-paste-1.0.0~rc.1-1.fc44.x86_64.rpm'
/usr/bin/nebula-paste --version
```

Expected: `Nebula Paste 1.0.0-rc.1`. DNF upgrades an existing 0.9 package. Add Nebula Paste in COSMIC panel applet settings. Log out and back in if the launcher has not refreshed. Check the running program:

```bash
for pid in $(pgrep -x nebula-paste); do readlink "/proc/$pid/exe"; done
```

Expected: `/usr/bin/nebula-paste`, without `(deleted)`. French/English OCR is bundled. Optional direct paste requires `wtype` and compositor support for virtual keyboards; copying followed by Ctrl+V remains available.

## Data and removal

History: `${XDG_DATA_HOME:-~/.local/share}/nebula-paste/history.sqlite3`. Preferences: `${XDG_CONFIG_HOME:-~/.config}/nebula-paste/settings.conf`. This candidate introduces no new database schema migration.

Before upgrading, export history using Preferences → Backup and diagnostics, and export templates separately. File references do not back up the referenced files. Exports and the database are not encrypted.

To uninstall, remove the applet, close its window and run `sudo dnf remove nebula-paste`. User data is retained. Downgrading does not guarantee future database compatibility; keep exports.

## Troubleshooting

- Blur: compare the popup with the calendar on the same background, then expand/reduce. COSMIC has separate window/applet settings. This fix covers popups; the separate history window still needs visual validation.
- Wrong version: check the running binary path and `rpm -q nebula-paste`; old local launchers or processes may remain.
- Missing history: use the same account and XDG directories; never launch the applet with sudo.
- Reports: include app and Fedora/COSMIC versions plus reproduction steps. Built-in diagnostics exclude clip content; inspect screenshots before sharing.
