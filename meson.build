project('smlsharp')
add_languages(['c','cpp'])
dep_llvm = dependency('llvm',version : '>=3.9')

# SML# runtime require atomic.  
cc = meson.get_compiler('c')
dep_atomic =cc.find_library('atomic')


dep_threads = dependency('threads',required:false)
dep_myth = cc.find_library('myth',required:false)
dep_m = cc.find_library('m')
dep_gmp =cc.find_library('gmp')
dep_socket = cc.find_library('socket',required:  false)
dep_dl = cc.find_library('dl')
dep_rt = cc.find_library('rt')
dep_unwind = cc.find_library('unwind',required: false)
llvm_version_number = dep_llvm.version() #
llvm_major_minor_patch = llvm_version_number.split('.')
llvm_major = llvm_major_minor_patch.get(0) # major version 

llvm_minor = llvm_major_minor_patch.get(1) # major version 

llvm_patch = llvm_major_minor_patch.get(2) # major version 
message('LLVM Version' ,llvm_major)
#--------------Check C Capability-----------------#

checkpassed = true

message('Checking C capability')

#Atomic check
message('Check Atomic')
config_h = configuration_data()

if cc.has_header('stdatomic.h')
    config_h.set('HAVE_STDATOMIC_H',1)
else 
    gccatomic_check = 'int foo(void **p1,void **p2, void *p3){return __atomic_compare_exchange_n(p1,p2,p3,1,__ATOMIC_RELEASE,__ATOMIC_ACQUIRE);}'
    if cc.compiles(gccatomic_check)
        config_h.set('HAVE_GCC_ATOMIC',1)
    else 
        error('__atomic builtins are not available. Use GCC 4.7 or later')
    endif
endif 
# alignof check
message('alignof check')
alignof_check= 'int x = alignof(void *);' 
if cc.compiles(alignof_check) 
    config_h.set('HAVE_ALIGNOF',1) 
else 
    if cc.compiles('int x = __alignof__(void *);')
        config_h.set('HAVE_ALIGNOF',1)
        config_h.set('alignof','__alignof__ ')
    else
        warning('alignof or __alignof__ is not supported')    
    endif
endif 

# tls check

message('TLS check')
tls_check = '_Thread_local int hoge; int foo(){ return hoge;}'

if cc.compiles(tls_check) 
    config_h.set('HAVE_TLS',1)
else 
    thread_check = '__thread int hoge ;int foo(){return hoge;}'
    if cc.compiles(thread_check)
        config_h.set('_Thread_local','__thread')
        config_h.set('HAVE_TLS',1)
    else 
        warning('_Thread_local or __thread is not supported.')
    endif
endif 

# check Headers
# C99 headers

if cc.has_header('assert.h') 
	 config_h.set('HAVE_ASSERT_H',1) 
 endif 
if cc.has_header('ctype.h') 
	 config_h.set('HAVE_CTYPE_H',1) 
 endif 
if cc.has_header('dirent.h') 
	 config_h.set('HAVE_DIRENT_H',1) 
 endif 
if cc.has_header('errno.h') 
	 config_h.set('HAVE_ERRNO_H',1) 
 endif 
if cc.has_header('inttypes.h') 
	 config_h.set('HAVE_INTTYPES_H',1) 
 endif 
if cc.has_header('limits.h') 
	 config_h.set('HAVE_LIMITS_H',1) 
 endif 
if cc.has_header('math.h') 
	 config_h.set('HAVE_MATH_H',1) 
 endif 
if cc.has_header('setjmp.h') 
	 config_h.set('HAVE_SETJMP_H',1) 
 endif 
if cc.has_header('stdarg.h') 
	 config_h.set('HAVE_STDARG_H',1) 
 endif 
if cc.has_header('stddef.h') 
	 config_h.set('HAVE_STDDEF_H',1) 
 endif 
if cc.has_header('stdint.h') 
	 config_h.set('HAVE_STDINT_H',1) 
 endif 
if cc.has_header('stdio.h') 
	 config_h.set('HAVE_STDIO_H',1) 
 endif 
if cc.has_header('stdlib.h') 
	 config_h.set('HAVE_STDLIB_H',1) 
 endif 
if cc.has_header('string.h') 
	 config_h.set('HAVE_STRING_H',1) 
 endif 
if cc.has_header('time.h') 
	 config_h.set('HAVE_TIME_H',1) 
 endif 

# POSIX headers
if cc.has_header('fcntl.h') 
	# config_h.set('HAVE_FCNTL_H',1) 
 endif 
if cc.has_header('fenv.h') 
	# config_h.set('HAVE_FENV_H',1) 
 endif 
if cc.has_header('dlfcn.h') 
	# config_h.set('HAVE_DLFCN_H',1) 
 endif 
if cc.has_header('poll.h') 
	# config_h.set('HAVE_POLL_H',1) 
 endif 
if cc.has_header('signal.h') 
	# config_h.set('HAVE_SIGNAL_H',1) 
 endif 
if cc.has_header('sys/mman.h') 
	 config_h.set('HAVE_SYS_MMAN_H',1) 
 endif 
if cc.has_header('sys/resource.h') 
	 config_h.set('HAVE_SYS_RESOURCE_H',1) 
 endif 
if cc.has_header('sys/time.h') 
	 config_h.set('HAVE_SYS_TIME_H',1) 
 endif 
if cc.has_header('sys/times.h') 
	 config_h.set('HAVE_SYS_TIMES_H',1) 
 endif 
if cc.has_header('sys/stat.h') 
	 config_h.set('HAVE_SYS_STAT_H',1) 
 endif 
if cc.has_header('sys/socket.h') 
	 config_h.set('HAVE_SYS_SOCKET_H',1) 
 endif 
if cc.has_header('sys/types.h')
    config_h.set('HAVE_SYS_TYPES_H',1)
endif
if cc.has_header('unistd.h') 
	 config_h.set('HAVE_UNISTD_H',1) 
 endif 
if cc.has_header('utime.h') 
	 config_h.set('HAVE_UTIME_H',1) 
 endif 
if cc.has_header('HAVE_SYS_TYPES_H')
    config_h.set('HAVE_SYS_TYPES_H',1)
endif 
if target_machine.system().contains('windows')
    cc.has_header('windows.h',required:true)
endif 
cc.has_header('libunwind.h')
if cc.has_function('ceilf') 
	 config_h.set('HAVE_CEILF',1) 
 endif 
if cc.has_function('clock_gettime') 
	 config_h.set('HAVE_CLOCK_GETTIME',1) 
 endif 
if cc.has_function('copysign') 
	 config_h.set('HAVE_COPYSIGN',1) 
 endif 
if cc.has_function('copysignf') 
	 config_h.set('HAVE_COPYSIGNF',1) 
 endif 
if cc.has_function('dlopen') 
	 config_h.set('HAVE_DLOPEN',1) 
 endif 
if cc.has_function('fegetround') 
	 config_h.set('HAVE_FEGETROUND',1) 
 endif 
if cc.has_function('fesetround') 
	 config_h.set('HAVE_FESETROUND',1) 
 endif 
if cc.has_function('finite') 
	 config_h.set('HAVE_FINITE',1) 
 endif 
if cc.has_function('floorf') 
	 config_h.set('HAVE_FLOORF',1) 
 endif 
if cc.has_function('fpclass') 
	 config_h.set('HAVE_FPCLASS',1) 
 endif 
if cc.has_function('frexpf') 
	 config_h.set('HAVE_FREXPF',1) 
 endif 
if cc.has_function('getrusage') 
	 config_h.set('HAVE_GETRUSAGE',1) 
 endif 
if cc.has_function('gettimeofday') 
	 config_h.set('HAVE_GETTIMEOFDAY',1) 
 endif 
if cc.has_function('ldexpf') 
	 config_h.set('HAVE_LDEXPF',1) 
 endif 
if cc.has_function('mkstemp') 
	 config_h.set('HAVE_MKSTEMP',1) 
 endif 
if cc.has_function('mmap') 
	 config_h.set('HAVE_MMAP',1) 
 endif 
if cc.has_function('modf') 
	 config_h.set('HAVE_MODF',1) 
 endif 
if cc.has_function('modff') 
	 config_h.set('HAVE_MODFF',1) 
 endif 
if cc.has_function('nextafter') 
	 config_h.set('HAVE_NEXTAFTER',1) 
 endif 
if cc.has_function('nextafterf') 
	 config_h.set('HAVE_NEXTAFTERF',1) 
 endif 
if cc.has_function('poll') 
	 config_h.set('HAVE_POLL',1) 
 endif 
if cc.has_function('readlink') 
	 config_h.set('HAVE_READLINK',1) 
 endif 
if cc.has_function('roundf') 
	 config_h.set('HAVE_ROUNDF',1) 
 endif 
if cc.has_function('select') 
	 config_h.set('HAVE_SELECT',1) 
 endif 
if cc.has_function('sleep') 
	 config_h.set('HAVE_SLEEP',1) 
 endif 
if cc.has_function('socket') 
	 config_h.set('HAVE_SOCKET',1) 
 endif 
if cc.has_function('strptime') 
	 config_h.set('HAVE_STRPTIME',1) 
 endif 
if cc.has_function('times') 
	 config_h.set('HAVE_TIMES',1) 
 endif 
if cc.has_function('utime') 
	 config_h.set('HAVE_UTIME',1) 
 endif 
if cc.has_function('utimes') 
	 config_h.set('HAVE_UTIMES',1) 
 endif 
#configure config.h
if dep_dl.found() 
    config_h.set('HAVE_LIBDL',1)
endif 
if dep_gmp.found() 
    config_h.set('HAVE_LIBGMP',1)
endif 
if dep_m.found() 
    config_h.set('HAVE_LIBM',1)
endif 
if dep_myth.found() 
    config_h.set('HAVE_LIBMYTH',1)
endif 
if dep_threads.found() 
    config_h.set('HAVE_LIBPTHREAD',1)
endif 
if dep_rt.found() 
    config_h.set('HAVE_LIBRT',1)
endif 
if dep_socket.found() 
    config_h.set('HAVE_LIBSOCKET',1)
endif 
if dep_unwind.found()
    config_h.set('HAVE_LIBUNWIND',1)
endif 

#check declaration 
if cc.run('''#include<time.h>
int main(int argc, char **argv) {
  if (CLOCK_MONOTONIC){  
    return 0;
  }else{
    return 1;
  }
}''').returncode() == 0
    config_h.set('HAVE_DECL_CLOCK_MONOTONIC',1,description:'''Define to 1 if you have the declaration of `CLOCK_MONOTONIC', and to 0 if
   you don't.''')
endif 
if cc.has_function('fpclassify',prefix:'#include<math.h>')
    config_h.set('HAVE_DECL_FPCLASSIFY',1,description : '''Define to 1 if you have the declaration of `fpclassify', and to 0 if you
   don't.''')
endif
if cc.has_function('isinf',prefix:'#include<math.h>')
    config_h.set('HAVE_DECL_ISINF',1)
endif
if cc.has_function('isnan',prefix:'#include<math.h>')
    config_h.set('HAVE_DECL_ISNAN',1)
endif
if cc.has_function('isnormal',prefix:'#include<math.h>')
    config_h.set('HAVE_DECL_ISNORMAL',1)
endif
if cc.has_function('signbit',prefix:'#include<math.h>')
    config_h.set('HAVE_DECL_SIGNBIT',1)
endif

if cc.has_type('max_align_t',prefix:'#include<stddef.h>')
    config_h.set('HAVE_MAX_ALIGN_T',1)
endif

if cc.sizeof('int') == 4 
    config_h.set('SIZEOF_INT',4)
endif 
if cc.sizeof('float') == 4 
    config_h.set('SIZEOF_FLOAT',4)
endif 
if cc.sizeof('double') == 8
    config_h.set('SIZEOF_DOUBLE',8)
endif 
sizeof_long = cc.sizeof('long')
if  sizeof_long == 4 or sizeof_long == 8
    config_h.set('SIZEOF_LONG',sizeof_long)
endif 
sizeof_ll = cc.sizeof('long long')
if sizeof_ll == 4 or sizeof_ll == 8
    config_h.set('SIZEOF_LONG_LONG',sizeof_ll)
endif 
sizeof_size_t = cc.sizeof('size_t')
if sizeof_size_t == 4 or sizeof_size_t ==8
    config_h.set('SIZEOF_SIZE_T',sizeof_size_t)
endif 

configure_file(output:'config.h',configuration:config_h,input : 'config.h.inmeson')




#--------------Configuration---------------------# 
PRECOMPILED_ARCH = 'x86_64'
SMLSHARP_ENV = 'SMLSHARP_HEAPSIZE=32M:2G'
#--------------TOOL chain------------------------#

# used to get llvm version.
llvm_config = find_program('llvm-config')

#use to compile bootstrap compiler. 
llc = find_program('llc', 'llc-'+llvm_major)
#use to compile bootstrap compiler 
llvm_as = find_program('llvm-as','llvm-as-'+llvm_major)
#for unix platform
xz = find_program('xz')
target_triple = run_command(
    [llvm_config, '--host-target'],check:true
).stdout().replace('\n', '').replace('\r', '')

subdir('src/llvm/main')
subdir('precompiled')
subdir('tests')
top_dir = meson.current_build_dir()
subdir('src/sql/main/builtintypes/timestamp')
subdir('src/runtime')

main_o = libsmlsharp.extract_objects('main.c')
minismlsharp_o = minismlsharp_
signal_o = compiler_support_objects.extract_objects('signal.c')
minismlsharp = executable(
    'minismlsharp',
    sources: [minismlsharp_o],
    objects: [main_o,signal_o],
    link_whole: smlsharp,
    link_depends: [minismlsharp_,],
    dependencies: [dep_m,dep_threads,dep_gmp,dep_myth],

)

# define SML# build command 

sml_compile = generator(minismlsharp,arguments: ['-Bsrc','-c'],output : ['@BASENAME@.o','@BASENAME@.smi'])

subdir('src/basis/main')
#sml_link = generator(minismlsharp,arguments:[])


if find_program('smlyacc',required : false).found()
    smlyacc = find_program('smlyacc')
else 
    smlyacc = executable('smlyacc',)
endif
if find_program('smllex',required : false).found()
    smllex = find_program('smllex',)
else 
    smllex = executable('smllex',)
endif
if find_program('smlformat',required : false).found()
    smlformat = find_program('smlformat')
else 
    smlformat = executable('smlformat')
endif

ppg_gen = generator(smlformat,arguments:['@INPUT@'],output: '@BASENAME@.ppg.sml')
