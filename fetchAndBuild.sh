#!/bin/sh

echo "This script attempts to fetch and build DGGAL, DGG and its bindings and examples for C, C++, Python and rust."
echo "Please make sure you have git installed to fetch the source code from the eC and DGGAL repositories."
echo "Please make sure you have zlib (dev package) installed, as well as GCC or Clang, and GNU Make."
echo "Please make sure you have GCC or Clang C++ support installed."
echo "Please make sure you have the rust compiler (rustc) edition 2024+ installed."
echo "Please make sure you have cffi installed for Python (pip3 install cffi)."
echo ""
echo "Building in 'dgbuild' directory..."

mkdir dgbuild
cd dgbuild

echo "Fetching eC core development environment..."
git clone -b main --single-branch https://github.com/ecere/eC.git

echo "Fetching DGGAL..."
git clone -b eC-core --single-branch https://github.com/ecere/dggal.git

echo "Building eC development environment..."
cd eC
make -j4

echo "Building DGGAL..."
cd ../dggal/
make -j4

echo ""
echo "**************************************"
echo "*********** DGGAL for rust ***********"
echo "**************************************"
echo "Building DGGAL for rust..."
cd bindings/rust
make
echo "Building DGGAL sample rust application..."
cd ../../bindings_examples/rust
make
echo "Execution test for DGGAL sample rust application:"
obj/linux/info ISEA3H A4-0-A
cd ../..

echo ""
echo "All done! Thank you for trying out and using DGGAL."
