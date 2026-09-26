# Install Nebula Paste 1.1.0-beta.1

[Français](INSTALL-1.1.md)

Beta for **Fedora 44 x86_64 / COSMIC**, unsigned upstream package. 1.0.0-rc.1 remains separately available. Export history and templates before testing. This version does not change the data schema; new preferences are additive.

Once [the release](https://github.com/nabz54/nebula-paste/releases/tag/v1.1.0-beta.1) is published, download and verify the binary without a GitHub account:

```bash
(
set -euo pipefail
directory=$(mktemp -d "$HOME/nebula-1.1-XXXXXX")
cd "$directory"
base='https://github.com/nabz54/nebula-paste/releases/download/v1.1.0-beta.1'
package='nebula-paste-1.1.0.beta.1-1.fc44.x86_64.rpm'
curl -fL "$base/$package" -o "$package"
curl -fL "$base/SHA256SUMS" -o SHA256SUMS
grep -F "  $package" SHA256SUMS | sha256sum -c -
sudo dnf install "./$package"
)
```

The filename uses dots; the internal RPM version remains `1.1.0~beta.1` for correct prerelease ordering. The manifest also lists a source RPM, which is not needed for installation.

Log out and back into COSMIC to restart the applet. Verify:

```bash
/usr/bin/nebula-paste --version
for pid in $(pgrep -x nebula-paste); do readlink "/proc/$pid/exe"; done
```

Expected: `Nebula Paste 1.1.0-beta.1` and `/usr/bin/nebula-paste`, without `(deleted)`. If a `~/.local/bin` installation shadows the RPM, follow [the migration guide](INSTALL.en.md).

In preferences, select the expanded shelf, card sizes, filter/collection visibility and opening collection. Follow [the 1.1 checks](TESTING-1.1.en.md). Rendering and focus still need confirmation in a real session; this is a beta.
