# Installing stl-thumb on Fedora (Wayland)

Guide for enabling STL, 3MF, and OBJ file thumbnails in Nautilus (GNOME Files) and Thunar on Fedora 39+ with Wayland.

## Why a separate guide?

GNOME on Wayland runs thumbnailers inside a bubblewrap (`bwrap`) sandbox with `--unshare-all`. The default `xvfb-run` wrapper in the shipped `.thumbnailer` files does not work in this sandbox because:

- SELinux blocks `xvfb-run` from executing `xauth` under the `thumb_t` domain
- The sandbox has no display server, causing winit to panic

The fix is to use OSMesa (Mesa's off-screen software renderer), which lets `stl-thumb` render without any display server. This works perfectly inside the sandbox.

## 1. Install dependencies

```bash
sudo dnf install mesa-compat-libOSMesa
```

The `osmesa-sys` crate loads `libOSMesa.so` by name at runtime, but the package only ships the versioned `libOSMesa.so.8`. Create the symlink:

```bash
sudo ln -sf /usr/lib64/libOSMesa.so.8 /usr/lib64/libOSMesa.so
```

## 2. Install stl-thumb

### Option A: From a release binary

Download the latest release from [GitHub Releases](https://github.com/unlimitedbacon/stl-thumb/releases) and copy the binary:

```bash
sudo cp stl-thumb /usr/bin/stl-thumb
sudo chmod 755 /usr/bin/stl-thumb
```

### Option B: Build from source

```bash
cargo build --release
sudo cp target/release/stl-thumb /usr/bin/stl-thumb
sudo chmod 755 /usr/bin/stl-thumb
```

**Important:** Install to `/usr/bin/`, not `/usr/local/bin/`. GNOME's bubblewrap sandbox mounts `/usr` read-only, and `/usr/local/bin` takes precedence in PATH. If an older binary exists in `/usr/local/bin`, the sandbox will use it instead of `/usr/bin`. Either install to `/usr/bin/` only, or make sure both locations have the same version.

## 3. Install thumbnailer configs

Do **not** use the `.thumbnailer` files from the repo as-is - they reference `xvfb-run` which fails in the sandbox. Create them with direct `stl-thumb` calls instead:

```bash
sudo tee /usr/share/thumbnailers/stl-thumb.thumbnailer > /dev/null << 'EOF'
[Thumbnailer Entry]
TryExec=stl-thumb
Exec=stl-thumb -f png -s %s %i %o
MimeType=model/3mf;model/stl;model/x.stl-ascii;model/x.stl-binary;application/sla;
EOF

sudo tee /usr/share/thumbnailers/obj-thumb.thumbnailer > /dev/null << 'EOF'
[Thumbnailer Entry]
TryExec=stl-thumb
Exec=stl-thumb -f png -s %s %i %o
MimeType=model/obj;
EOF
```

## 4. Configure your file manager

### Nautilus (GNOME Files)

Nautilus has a thumbnail file size limit (default 50 MB). Large STL files may exceed this. Raise it to 1 GB:

```bash
gsettings set org.gnome.nautilus.preferences thumbnail-limit 1000
```

### Thunar (Xfce)

Thunar uses tumblerd, which picks up `.thumbnailer` files automatically. The default `MaxFileSize=0` in `/etc/xdg/tumbler/tumbler.rc` means no size limit.

## 5. Activate

Clear the thumbnail cache (including any cached failures from previous attempts) and restart your file manager:

```bash
rm -rf ~/.cache/thumbnails/*

# For Nautilus
nautilus -q

# For Thunar
killall tumblerd
```

Open a folder containing STL/3MF/OBJ files and thumbnails should generate automatically.

## Troubleshooting

### Thumbnails not appearing

Check for cached failures. GNOME caches failed thumbnail attempts and will not retry them:

```bash
ls ~/.cache/thumbnails/fail/gnome-thumbnail-factory/
```

If there are entries, clear them and restart:

```bash
rm -rf ~/.cache/thumbnails/fail/*
nautilus -q
```

### SELinux denials

If you see SELinux-related errors (only relevant if using `xvfb-run`, which this guide avoids):

```bash
sudo ausearch -m avc -ts recent | grep thumb_t
```

The OSMesa approach in this guide does not trigger SELinux denials.

### Specific files not thumbnailing

Some 3MF files may fail to render. Test a specific file to confirm:

```bash
stl-thumb -f png -s 256 yourfile.3mf /tmp/test.png
```

If this errors, the file may use 3MF features that are not yet supported.

### Verifying the sandbox works

You can test that `stl-thumb` works inside a bubblewrap sandbox identical to GNOME's:

```bash
mkdir -p /tmp/thumb-test
bwrap --ro-bind /usr /usr \
  --ro-bind-try /etc/ld.so.cache /etc/ld.so.cache \
  --symlink /usr/bin /bin \
  --symlink /usr/lib64 /lib64 \
  --symlink /usr/lib /lib \
  --symlink /usr/sbin /sbin \
  --proc /proc --dev /dev --chdir / \
  --unshare-all --die-with-parent \
  --bind /tmp/thumb-test /tmp \
  --ro-bind yourfile.stl /tmp/input.stl \
  stl-thumb -f png -s 256 /tmp/input.stl /tmp/output.png

ls -la /tmp/thumb-test/output.png
```

Exit code 0 and a valid PNG means everything is working.
