PRECOMPILED_ARCH = 'x86_64'
STAGE0_SRC_AR = PRECOMPILED_ARCH + '.ll.xz'
STAGE0_SRC = PRECOMPILED_ARCH + '.ll'
STAGE0_BITCODE = PRECOMPILED_ARCH + '.bc'
STAGE0_OBJ = PRECOMPILED_ARCH + '.o'
llvmir = custom_target(command: [xz, '-k','-f','-d','@INPUT@'], output: STAGE0_SRC,input:STAGE0_SRC_AR)
llvmbitcode = custom_target(
    command: [llvm_as, '-o=precompiled/' + STAGE0_BITCODE, '../@INPUT@'],
    input : llvmir ,
    output: STAGE0_BITCODE,
)
LLCFLAGS =  []
cpu_family = target_machine.cpu_family()
if cpu_family.contains('x86')
    LLCFLAGS += '-no-x86-call-frame-opt'

else
    error('target CPU is not supported')
endif 

if target_machine.cpu_family().contains('x86_64')
 LLCFLAGS +='-relocation-model=pic'
endif 
bitcode_path = llvmbitcode.full_path()
minismlsharp_ = custom_target(
    command: [
        llc,
        '-load=' + llvm_plugin.full_path(),
        '-mtriple=' + target_triple,
        '-O2',
        '-tailcallopt',
        '-filetype=obj',
        '-o=@OUTPUT@',
    ] + LLCFLAGS +  
        bitcode_path
    ,
    output: STAGE0_OBJ,
    input : llvmbitcode
)
#