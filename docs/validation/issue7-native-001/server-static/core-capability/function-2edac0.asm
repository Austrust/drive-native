_ZN9synodrive4core14NodeCapability18NodeCapabilityImpl21GetUiAndSharePrivFuncB5cxx11ERKNS1_5ParamE
2edac0: endbr64  ; 
2edac4: pxor xmm0, xmm0 ; 
2edac8: lea rcx, [rip - 0x306f] ; 
2edacf: mov rax, rdi ; 
2edad2: movups xmmword ptr [rdi], xmm0 ; 
2edad5: movq xmm0, rcx ; 
2edada: mov qword ptr [rdi], rdx ; 
2edadd: lea rdx, [rip + 0x9fc] ; 
2edae4: movq xmm1, rdx ; 
2edae9: punpcklqdq xmm0, xmm1 ; 
2edaed: movups xmmword ptr [rdi + 0x10], xmm0 ; 
2edaf1: ret  ; 
