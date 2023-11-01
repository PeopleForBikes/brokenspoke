# Lambdas

## Cross Compiling

In case the cross compilation fails due to not finding the right Python
interpreter for Linux on the Mac machine, try the following steps.

```bash
cd /tmp
mdir libpython
cd libpython
curl -lO http://http.us.debian.org/debian/pool/main/p/python3.11/libpython3.11_3.11.6-3_amd64.deb
ar x libpython3.11_3.11.6-3_amd64.deb
tar xvJf data.tar.xz
ln usr/lib/x86_64-linux-gnu/libpython3.11.so.1.0 ~/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/x86_64-unknown-linux-gnu/lib/libpython3.11.so
```

Make sure to export the Python version:

```bash
export PYO3_CROSS_PYTHON_VERSION=3.11
```

Then compile again:

```bash
PYO3_CROSS_PYTHON_VERSION=3.11 cargo lambda build --release
```

And it works!

```bash
$ file target/lambda/lambda-city/bootstrap
target/lambda/lambda-city/bootstrap: ELF 64-bit LSB pie executable, x86-64, version 1 (SYSV), dynamically linked, interpreter /lib64/ld-linux-x86-64.so.2, for GNU/Linux 2.0.0, stripped
```
