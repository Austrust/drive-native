_ZNO8DriveAcl10Capability9ToPObjectEv
257450: endbr64  ; 
257454: push rbx ; 
257455: mov rbx, rdi ; 
257458: sub rsp, 0x10 ; 
25745c: mov rax, qword ptr fs:[0x28] ; 
257465: mov qword ptr [rsp + 8], rax ; 
25746a: xor eax, eax ; 
25746c: call 0x256ea0 ; _ZNKR8DriveAcl10Capability9ToPObjectEv
257471: mov rax, qword ptr [rsp + 8] ; 
257476: sub rax, qword ptr fs:[0x28] ; 
25747f: jne 0x25748a ; 
257481: add rsp, 0x10 ; 
257485: mov rax, rbx ; 
257488: pop rbx ; 
257489: ret  ; 
25748a: call 0x1aac10 ; PLT:__stack_chk_fail
