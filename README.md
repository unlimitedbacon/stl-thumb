# stl-thumb

[![Build Status](https://travis-ci.org/unlimitedbacon/stl-thumb.svg?branch=master)](https://travis-ci.org/unlimitedbacon/stl-thumb)

stl-thumb is a fast lightweight thumbnail generator for STL files. It is written in Rust and uses OpenGL. It works with file managers on Linux and Windows.

![Screenshot](https://user-images.githubusercontent.com/3131268/42529042-31d9bca6-8432-11e8-9ba8-87d9b72aaddb.png)

## Installation

### Windows

1. Install the [Microsoft Visual C++ 2015 Redistributable](https://www.microsoft.com/en-us/download/details.aspx?id=52685). There is a good chance you already have this.
2. Download the MSI package from the Releases and install it.

The installer will tell the Windows shell to refresh the thumbnail cache, however this does not always seem to work. If your icons do not change then try using the Disk Cleanup utility to clear the thumbnail cache.

### Linux

Packages are being worked on for Arch and Debian/Ubuntu. Make sure your file manager is set to create previews for files > 1 Mb.

## Command Line Usage

```
$ stl-thumb <STL_FILE> [IMG_FILE]
```

### Options

| Option      | Description                                             |
| ----------- | ------------------------------------------------------- |
| -s \<size\> | Specify width of the image. It will always be a square. |
| -x          | Show a preview.                                         |
