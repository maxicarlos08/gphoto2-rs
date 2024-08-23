#!/bin/bash
set -e

echo cargo:rerun-if-changed=$0

root_dir=$PWD

cd $OUT_DIR

# get a Unix-y path in case original OUT_DIR was a Windows path and we are inside MSYS
OUT_DIR=$PWD

if [ ! -d libgphoto2 ] ; then
	mkdir libgphoto2
	curl -L https://github.com/maxicarlos08/libgphoto2/archive/refs/heads/maxicarlos08/fix-windows-vcamera-build.tar.gz | tar -xzv --strip 1 -C libgphoto2
fi

cd libgphoto2

if [ ! -f configure ] ; then
	autoreconf -iv
fi

# Set VCAMERADIR to an isolated local directory instead of the global /usr/... default.
vcamera_dir=$OUT_DIR/vcamera

mkdir -p $vcamera_dir/{foo,bar/baz}
cp $root_dir/blank.jpg $vcamera_dir
cp $root_dir/blank.jpg $vcamera_dir/foo
cp $root_dir/blank.jpg $vcamera_dir/bar

# Minimal build with just the virtual vusb driver.
./configure -C \
	--prefix=$OUT_DIR/install \
	--enable-vusb --with-camlibs=ptp2 \
	--disable-serial --disable-ptpip --disable-disk --without-libusb-1.0 --without-libusb \
	--disable-nls \
	--without-libexif --without-jpeg --without-libxml-2.0 --without-libcurl --without-gdlib \
	CFLAGS="-g"

export MAKEFLAGS=$CARGO_MAKEFLAGS

# Unfortunately, MSYS/MINGW make can't handle the Cargo jobserver args.
# But, mingw32-make can.
case "$(uname -s)" in
CYGWIN*|MINGW32*|MSYS*|MINGW*)
	mingw32-make install
	;;
*)
	make install
	;;
esac
