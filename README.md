# stl-thumb

[![Build Status](https://travis-ci.org/unlimitedbacon/stl-thumb.svg?branch=master)](https://travis-ci.org/unlimitedbacon/stl-thumb)

Stl-thumb is a fast lightweight thumbnail generator for STL files. It can show previews for STL files in your file manager on Linux and Windows. It is written in Rust and uses OpenGL.

![Screenshot](https://user-images.githubusercontent.com/3131268/42529042-31d9bca6-8432-11e8-9ba8-87d9b72aaddb.png)

## Installation

### Windows

Stl-thumb requires 64 bit Windows Vista or later. [Download the installer .exe](https://github.com/unlimitedbacon/stl-thumb/releases/latest) for the latest release and run it.

The installer will tell the Windows shell to refresh the thumbnail cache, however this does not always seem to work. If your icons do not change then try using the [Disk Cleanup](https://en.wikipedia.org/wiki/Disk_Cleanup) utility to clear the thumbnail cache.

### Linux

Make sure that your file manager is set to generate previews for files larger than 1 MB. Most file managers have this setting under the Preview tab in their Preferences.

Stl-thumb does not currently work with Nautilus (Gnome Files) 3.26. This is the default file manager in Ubuntu 18.04. [Nautilus sandboxes the thumbnailers](https://gitlab.gnome.org/GNOME/gnome-desktop/issues/7), preventing us from using GL. For now you will have to use a different file manager instead.

#### Arch

A package is available [in the AUR](https://aur.archlinux.org/packages/stl-thumb/). Install it manually or using your favorite AUR helper.

```
$ yay -S stl-thumb
```

#### Debian / Ubuntu

[Download the .deb package](https://github.com/unlimitedbacon/stl-thumb/releases/latest) for your platform (usually amd64) and install it. Packages are also available for armhf (Raspberry Pi) and arm64.

```
$ sudo dpkg -i stl-thumb_0.1.0_amd64.deb
```

## Command Line Usage

```
$ stl-thumb <STL_FILE> [IMG_FILE]
```

### Options

| Option        | Description                                             |
| ------------- | ------------------------------------------------------- |
| <STL_FILE>    | The STL file you want a picture of.                     |
| [IMG_FILE]    | The thumbnail image file that will be created. Always PNG format. If this is omitted, the image data will be dumped to stdout. |
| -s \<size\>   | Specify width of the image. It will always be a square. |
| -x            | Display the image in a window.                          |
| -h, --help    | Prints help information.                                |
| -V, --version | Prints version information.                             |
| -v[v][v]      | Increase message verbosity.                             |
