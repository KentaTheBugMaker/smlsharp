use cc::Build;

extern crate cc;
/*
CFLAGS = -g -O2 -fPIC
CXXFLAGS = -g -O2
DEFS = -DNDEBUG -DHAVE_CONFIG_H  -DHOST_CPU_i386
NETLIB_CFLAGS = $(CFLAGS) -DIEEE_8087 -DMALLOC=sml_xmalloc -DLong=int
CPPFLAGS = -I. 
RDYNAMIC_SMLFLAGS = -Xlinker -rdynamic
LLCFLAGS =  -no-x86-call-frame-opt -relocation-model=pic
LLVM_CXXFLAGS = -I/usr/lib/llvm-18/include -std=c++17   -fno-exceptions -funwind-tables -D_GNU_SOURCE -D__STDC_CONSTANT_MACROS -D__STDC_FORMAT_MACROS -D__STDC_LIMIT_MACROS
LLVM_VERSION = 18.1
*/
fn main(){
    // compile netlib separately
    //$ cc -g -O2 -fPIC -DIEEE_8087 -DMALLOC=sml_xmalloc -DLong=int -c ../src/runtime/netlib/dtoa.c -o ../src/runtime/netlib/dtoa.o
    //$ cc -g -O2 -fPIC -DIEEE_8087 -DMALLOC=sml_xmalloc -Dlong=int -c ../src/runtime/netlib/dtoa.c
    //$ cc -g -O2 -fPIC -DIEEE_8087 -DMALLOC=sml_xmalloc -DLong=int -c ../src/runtime/netlib/dtoa.c -o ../src/runtime/netlib/dtoa.o
    let dtoa = cc::Build::new()
    .flag("-g")
    .flag("-O2")
    .include("../")
    .include("../src/runtime")
    .define("IEEE_8087", None)
    .define("MALLOC", "sml_xmalloc")
    .define("Long", "int")
    .file("../src/runtime/netlib/dtoa.c")
    .compile_intermediates();
    //

    cc::Build::new()
    .flag("-g")
    .flag("-O2")
    .flag("-fPIC")
    .define("NDEBUG", None)
    .define("HAVE_CONFIG_H", None)
    .define("HOST_CPU_i386", None)
    .include("../")
    .include("../src/runtime")
    .file("../src/runtime/callback.c")
    .file("../src/runtime/control.c")
    .file("../src/runtime/dbglog.c")
    .file("../src/runtime/exn.c")
    .file("../src/runtime/finalize.c")
    .file("../src/runtime/heap_concurrent.c")
    .file("../src/runtime/init.c")
    .file("../src/runtime/object.c")
    .file("../src/runtime/prim.c")
    .file("../src/runtime/splay.c")
    .file("../src/runtime/top.c")
    .file("../src/runtime/xmalloc.c")
    .file("../src/runtime/livecheck.c")
    .object("../src/runtime/call_with_cleanup.o")
    .objects(dtoa)

    .compile("smlsharp-cblob")
}