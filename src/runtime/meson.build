
#compiler defines

cargs =['-DNDEBUG','-DHAVE_CONFIG_H','-DHOST_CPU_i386']
dtoa_cargs =['-DIEEE_8087','-DMALLOC=sml_xmalloc','-DLong=int']

call_with_cleanup = custom_target(
    command: [
        llc,
        '-mtriple=' + target_triple,
        '-O2',
        '-filetype=obj',]+
        LLCFLAGS
        +[
        '-o=@OUTPUT@',
        '@INPUT@',]
    ,
    input :'call_with_cleanup.ll',
    output : 'call_with_cleanup.o'
)

dtoa = static_library('dtoa','netlib/dtoa.c',c_args: dtoa_cargs,pic: true)
config_h_include = include_directories('../../')
runtime = static_library('runtime',sources: [
    'callback.c',
    'control.c',
    'error.c',
    'exn.c',
    'finalize.c',
    'init.c',
    'object.c',
    'xmalloc.c',
    'prim.c',
    'splay.c',
    'top.c',
    'heap_concurrent.c',
    ],
    link_whole: timestamp,
    c_args:cargs,
    include_directories:config_h_include,
dependencies: [dep_atomic,dep_threads,dep_gmp,dep_m])

smlsharp = static_library('smlsharp',[
    call_with_cleanup,
],
c_args:cargs,
link_whole: [dtoa,runtime],
include_directories: config_h_include,
)

libsmlsharp = static_library('libsmlsharp',sources:'main.c',
c_args:cargs,
include_directories: config_h_include,
dependencies: [dep_atomic,dep_threads]
)

compiler_support_objects = static_library('compiler_support','signal.c',include_directories: include_directories('../../'),
c_args: cargs,
dependencies:[dep_atomic,dep_threads])

#