_ZN13CreateACLTest11HandleErrorEv
223140: endbr64  ; 
223144: push r12 ; 
223146: push rbp ; 
223147: push rbx ; 
223148: sub rsp, 0x40 ; 
22314c: mov rdx, qword ptr [rdi + 8] ; 
223150: mov rax, qword ptr fs:[0x28] ; 
223159: mov qword ptr [rsp + 0x38], rax ; 
22315e: xor eax, eax ; 
223160: mov eax, dword ptr [rdx + 0x34c] ; 
223166: cmp eax, -3 ; 
223169: je 0x223258 ; 
22316f: test eax, eax ; 
223171: je 0x223234 ; 
223177: cmp eax, -4 ; 
22317a: je 0x2231b0 ; 
22317c: mov rax, qword ptr [rsp + 0x38] ; 
223181: sub rax, qword ptr fs:[0x28] ; 
22318a: jne 0x2232ce ; 
223190: mov rdi, qword ptr [rdx + 8] ; 
223194: add rsp, 0x40 ; 
223198: lea rcx, [rdx + 0x350] ; 
22319f: xor edx, edx ; 
2231a1: pop rbx ; 
2231a2: mov esi, 0x7000 ; 
2231a7: pop rbp ; 
2231a8: pop r12 ; 
2231aa: jmp 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2231af: nop  ; 
2231b0: lea rbp, [rsp + 0x10] ; 
2231b5: mov r12, qword ptr [rdx + 8] ; 
2231b9: lea rsi, [rsp + 8] ; 
2231be: xor edx, edx ; 
2231c0: mov rdi, rbp ; 
2231c3: lea rbx, [rsp + 0x20] ; 
2231c8: mov qword ptr [rsp + 8], 0x13 ; 
2231d1: mov qword ptr [rsp + 0x10], rbx ; 
2231d6: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
2231db: mov rdx, qword ptr [rsp + 8] ; 
2231e0: mov rcx, rbp ; 
2231e3: mov rdi, r12 ; 
2231e6: movdqa xmm0, xmmword ptr [rip + 0x48cad2] ; no write permisssion when deleteno delete permisno permission ton to access nodeaccess but no encannot get view no permission foon for this pathpath is not exisblocked by disabdisable downloadsion when downlodownload but no no preview permiion for file reqno read permissifailed to get vifailed to query admin-test-phase.cpp.o
2231ee: mov qword ptr [rsp + 0x10], rax ; 
2231f3: mov esi, 0x9004 ; 
2231f8: mov qword ptr [rsp + 0x20], rdx ; 
2231fd: movups xmmword ptr [rax], xmm0 ; 
223200: mov dword ptr [rax + 0xf], 0x6e6f6973 ; 
223207: mov rax, qword ptr [rsp + 0x10] ; 
22320c: mov qword ptr [rsp + 0x18], rdx ; 
223211: mov byte ptr [rax + rdx], 0 ; 
223215: xor edx, edx ; 
223217: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
22321c: mov rdi, qword ptr [rsp + 0x10] ; 
223221: cmp rdi, rbx ; 
223224: je 0x223234 ; 
223226: mov rax, qword ptr [rsp + 0x20] ; 
22322b: lea rsi, [rax + 1] ; 
22322f: call 0xea4a0 ; PLT:_ZdlPvm
223234: mov rax, qword ptr [rsp + 0x38] ; 
223239: sub rax, qword ptr fs:[0x28] ; 
223242: jne 0x2232ce ; 
223248: add rsp, 0x40 ; 
22324c: pop rbx ; 
22324d: pop rbp ; 
22324e: pop r12 ; 
223250: ret  ; 
223251: nop dword ptr [rax] ; 
223258: lea rbp, [rsp + 0x10] ; 
22325d: mov r12, qword ptr [rdx + 8] ; 
223261: lea rsi, [rsp + 8] ; 
223266: xor edx, edx ; 
223268: mov rdi, rbp ; 
22326b: lea rbx, [rsp + 0x20] ; 
223270: mov qword ptr [rsp + 8], 0x20 ; 
223279: mov qword ptr [rsp + 0x10], rbx ; 
22327e: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
223283: mov rdx, qword ptr [rsp + 8] ; 
223288: mov rcx, rbp ; 
22328b: mov rdi, r12 ; 
22328e: movdqa xmm0, xmmword ptr [rip + 0x48c9ba] ; no access permission when sharinserver operation failed, please refer to server sion for file resion when createno write permisssion when deleteno delete permisno permission ton to access nodeaccess but no encannot get view no permission foon for this pathpath is not exisblocked by disabdisable downloadsion when downlodownload but no no preview permiion for file reqno read permissifailed to get vi
223296: mov qword ptr [rsp + 0x10], rax ; 
22329b: mov esi, 0x9003 ; 
2232a0: mov qword ptr [rsp + 0x20], rdx ; 
2232a5: movups xmmword ptr [rax], xmm0 ; 
2232a8: movdqa xmm0, xmmword ptr [rip + 0x48ca00] ; sion when createno write permisssion when deleteno delete permisno permission ton to access nodeaccess but no encannot get view no permission foon for this pathpath is not exisblocked by disabdisable downloadsion when downlodownload but no no preview permiion for file reqno read permissifailed to get vifailed to query admin-test-phase.cpp.o
2232b0: movups xmmword ptr [rax + 0x10], xmm0 ; 
2232b4: mov rax, qword ptr [rsp + 0x10] ; 
2232b9: mov qword ptr [rsp + 0x18], rdx ; 
2232be: mov byte ptr [rax + rdx], 0 ; 
2232c2: xor edx, edx ; 
2232c4: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2232c9: jmp 0x22321c ; 
2232ce: call 0xead00 ; PLT:__stack_chk_fail
2232d3: endbr64  ; 
2232d7: mov rbp, rax ; 
2232da: jmp 0x1040de ; 
2232df: endbr64  ; 
2232e3: mov rbp, rax ; 
2232e6: jmp 0x1040de ; 
