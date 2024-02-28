# Re:Earth Flow Worker

## Development

### Install toolchains
- Rust (stable)

### Install prerequisites

```console
cargo install cargo-make
cargo install cargo-watch
```

### Linux/Debian

On linux systems you'd need the development headers of libxml2 (e.g. `libxml2-dev` in Debian), as well as `pkg-config`.

### MacOS
```
$ brew install libxml2 # e.g. version 2.9.12
$ ln -s /usr/local/Cellar/libxml2/2.9.12/lib/libxml2.2.dylib /usr/local/lib/libxml-2.0.dylib
$ export LIBXML2=/usr/local/Cellar/libxml2/2.9.12/lib/pkgconfig/libxml-2.0.pc
```

### Windows
* manually install builds tools c++ and english language by visiting [BuildTools](https://visualstudio.microsoft.com/fr/thank-you-downloading-visual-studio/?sku=BuildTools&rel=16)
* launch cmd prompt with admin privileges and execute these commands sequentially:

```
C:\> git clone https://github.com/microsoft/vcpkg
C:\> .\vcpkg\bootstrap-vcpkg.bat
C:\> setx /M PATH "%PATH%;c:\vcpkg" && setx VCPKGRS_DYNAMIC "1" /M
C:\> refreshenv
C:\> vcpkg install libxml2:x64-windows
C:\> vcpkg integrate install
```
