llvm_headers = run_command([llvm_config,'--includedir'],check :true).stdout().replace('\r', '').replace('\n', '')

llvm_plugin = shared_library('smlsharp_gc',
 'smlsharp_gc.cpp',
include_directories: [llvm_headers],
dependencies: [dep_llvm])

# anonymizer
anonymize = executable('anonymize',
'anonymize.cpp',
include_directories: [llvm_headers],
dependencies: [dep_llvm])
