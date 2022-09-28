#!/bin/bash
set -e

echo cargo:rerun-if-changed=$0

root_dir=$(dirname $(readlink -f $0))

cd $OUT_DIR

if [ ! -d libgphoto2 ] ; then
	mkdir libgphoto2
	curl -L https://github.com/gphoto/libgphoto2/archive/7605d4ab2b65d209bc94b6ae3fd0d26daf14a3f2.tar.gz | tar -xzv --strip 1 -C libgphoto2
fi

cd libgphoto2/libgphoto2_port

if [ ! -f configure ] ; then
	autoreconf -iv
fi

# Set VCAMERADIR to an isolated local directory instead of the global /usr/... default.
vcamera_dir=${OUT_DIR//\\/\/}/vcamera

mkdir -p $vcamera_dir/{foo,bar/baz}
cp $root_dir/blank.jpg $vcamera_dir/
cp $root_dir/blank.jpg $vcamera_dir/foo
cp $root_dir/blank.jpg $vcamera_dir/bar

# Minimal build with just the virtual vusb driver.
./configure -C \
	--enable-vusb \
	--disable-serial --disable-ptpip --disable-disk --without-libusb-1.0 --without-libusb --without-libexif \
	CFLAGS="-g"


# Unfortunately, MSYS/MINGW make can't handle the Cargo jobserver args.
case "$(uname -s)" in
CYGWIN*|MINGW32*|MSYS*|MINGW*)
	echo "Ignoring jobserver args: $CARGO_MAKEFLAGS"
	# Best-effort alternative - just pass number of available CPUs.
	make -j$(nproc)
	;;
*)
	MAKEFLAGS=$CARGO_MAKEFLAGS make
	;;
esac
