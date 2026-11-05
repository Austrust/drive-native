_ZN13RemoveACLTest11HandleErrorEv
2232f0: endbr64  ; 
2232f4: push r12 ; 
2232f6: push rbp ; 
2232f7: push rbx ; 
2232f8: sub rsp, 0x40 ; 
2232fc: mov rdx, qword ptr [rdi + 8] ; 
223300: mov rax, qword ptr fs:[0x28] ; 
223309: mov qword ptr [rsp + 0x38], rax ; 
22330e: xor eax, eax ; 
223310: mov eax, dword ptr [rdx + 0x34c] ; 
223316: cmp eax, -3 ; 
223319: je 0x223460 ; 
22331f: test eax, eax ; 
223321: je 0x2233c7 ; 
223327: cmp eax, -5 ; 
22332a: je 0x2233e8 ; 
223330: lea rbp, [rsp + 0x10] ; 
223335: mov r12, qword ptr [rdx + 8] ; 
223339: lea rsi, [rsp + 8] ; 
22333e: xor edx, edx ; 
223340: mov rdi, rbp ; 
223343: lea rbx, [rsp + 0x20] ; 
223348: mov qword ptr [rsp + 8], 0x33 ; 
223351: mov qword ptr [rsp + 0x10], rbx ; 
223356: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
22335b: mov rdx, qword ptr [rsp + 8] ; 
223360: mov rcx, rbp ; 
223363: mov rdi, r12 ; 
223366: movdqa xmm0, xmmword ptr [rip + 0x48c902] ; server operation failed, please refer to server sion for file resion when createno write permisssion when deleteno delete permisno permission ton to access nodeaccess but no encannot get view no permission foon for this pathpath is not exisblocked by disabdisable downloadsion when downlodownload but no no preview permiion for file reqno read permissifailed to get vifailed to query admin-test-phase
22336e: mov qword ptr [rsp + 0x10], rax ; 
223373: mov esi, 0x7000 ; 
223378: mov qword ptr [rsp + 0x20], rdx ; 
22337d: movups xmmword ptr [rax], xmm0 ; 
223380: movdqa xmm0, xmmword ptr [rip + 0x48c8f8] ;  failed, please refer to server sion for file resion when createno write permisssion when deleteno delete permisno permission ton to access nodeaccess but no encannot get view no permission foon for this pathpath is not exisblocked by disabdisable downloadsion when downlodownload but no no preview permiion for file reqno read permissifailed to get vifailed to query admin-test-phase.cpp.o
223388: movups xmmword ptr [rax + 0x10], xmm0 ; 
22338c: movdqa xmm0, xmmword ptr [rip + 0x48c8fc] ; refer to server sion for file resion when createno write permisssion when deleteno delete permisno permission ton to access nodeaccess but no encannot get view no permission foon for this pathpath is not exisblocked by disabdisable downloadsion when downlodownload but no no preview permiion for file reqno read permissifailed to get vifailed to query admin-test-phase.cpp.o
223394: movups xmmword ptr [rax + 0x20], xmm0 ; 
223398: mov dword ptr [rax + 0x2f], 0x676f6c20 ; 
22339f: mov qword ptr [rsp + 0x18], rdx ; 
2233a4: mov byte ptr [rax + rdx], 0 ; 
2233a8: xor edx, edx ; 
2233aa: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2233af: mov rdi, qword ptr [rsp + 0x10] ; 
2233b4: cmp rdi, rbx ; 
2233b7: je 0x2233c7 ; 
2233b9: mov rax, qword ptr [rsp + 0x20] ; 
2233be: lea rsi, [rax + 1] ; 
2233c2: call 0xea4a0 ; PLT:_ZdlPvm
2233c7: mov rax, qword ptr [rsp + 0x38] ; 
2233cc: sub rax, qword ptr fs:[0x28] ; 
2233d5: jne 0x2234d6 ; 
2233db: add rsp, 0x40 ; 
2233df: pop rbx ; 
2233e0: pop rbp ; 
2233e1: pop r12 ; 
2233e3: ret  ; 
2233e4: nop dword ptr [rax] ; 
2233e8: lea rbp, [rsp + 0x10] ; 
2233ed: mov r12, qword ptr [rdx + 8] ; 
2233f1: lea rsi, [rsp + 8] ; 
2233f6: xor edx, edx ; 
2233f8: mov rdi, rbp ; 
2233fb: lea rbx, [rsp + 0x20] ; 
223400: mov qword ptr [rsp + 8], 0x14 ; 
223409: mov qword ptr [rsp + 0x10], rbx ; 
22340e: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
223413: mov rdx, qword ptr [rsp + 8] ; 
223418: mov rcx, rbp ; 
22341b: mov rdi, r12 ; 
22341e: movdqa xmm0, xmmword ptr [rip + 0x48c8ba] ; no delete permisno permission ton to access nodeaccess but no encannot get view no permission foon for this pathpath is not exisblocked by disabdisable downloadsion when downlodownload but no no preview permiion for file reqno read permissifailed to get vifailed to query admin-test-phase.cpp.o
223426: mov qword ptr [rsp + 0x10], rax ; 
22342b: mov esi, 0xa004 ; 
223430: mov qword ptr [rsp + 0x20], rdx ; 
223435: mov dword ptr [rax + 0x10], 0x6e6f6973 ; 
22343c: movups xmmword ptr [rax], xmm0 ; 
22343f: mov rax, qword ptr [rsp + 0x10] ; 
223444: mov qword ptr [rsp + 0x18], rdx ; 
223449: mov byte ptr [rax + rdx], 0 ; 
22344d: xor edx, edx ; 
22344f: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
223454: jmp 0x2233af ; 
223459: nop dword ptr [rax] ; 
223460: lea rbp, [rsp + 0x10] ; 
223465: mov r12, qword ptr [rdx + 8] ; 
223469: lea rsi, [rsp + 8] ; 
22346e: xor edx, edx ; 
223470: mov rdi, rbp ; 
223473: lea rbx, [rsp + 0x20] ; 
223478: mov qword ptr [rsp + 8], 0x20 ; 
223481: mov qword ptr [rsp + 0x10], rbx ; 
223486: call 0xec1e0 ; PLT:_ZNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEE9_M_createERmm
22348b: mov rdx, qword ptr [rsp + 8] ; 
223490: mov rcx, rbp ; 
223493: mov rdi, r12 ; 
223496: movdqa xmm0, xmmword ptr [rip + 0x48c7b2] ; no access permission when sharinserver operation failed, please refer to server sion for file resion when createno write permisssion when deleteno delete permisno permission ton to access nodeaccess but no encannot get view no permission foon for this pathpath is not exisblocked by disabdisable downloadsion when downlodownload but no no preview permiion for file reqno read permissifailed to get vi
22349e: mov qword ptr [rsp + 0x10], rax ; 
2234a3: mov esi, 0xa003 ; 
2234a8: mov qword ptr [rsp + 0x20], rdx ; 
2234ad: movups xmmword ptr [rax], xmm0 ; 
2234b0: movdqa xmm0, xmmword ptr [rip + 0x48c818] ; sion when deleteno delete permisno permission ton to access nodeaccess but no encannot get view no permission foon for this pathpath is not exisblocked by disabdisable downloadsion when downlodownload but no no preview permiion for file reqno read permissifailed to get vifailed to query admin-test-phase.cpp.o
2234b8: movups xmmword ptr [rax + 0x10], xmm0 ; 
2234bc: mov rax, qword ptr [rsp + 0x10] ; 
2234c1: mov qword ptr [rsp + 0x18], rdx ; 
2234c6: mov byte ptr [rax + rdx], 0 ; 
2234ca: xor edx, edx ; 
2234cc: call 0x553b20 ; _ZN8Response8SetErrorEmmRKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
2234d1: jmp 0x2233af ; 
2234d6: call 0xead00 ; PLT:__stack_chk_fail
2234db: endbr64  ; 
2234df: mov rbp, rax ; 
2234e2: jmp 0x1040fe ; 
2234e7: endbr64  ; 
2234eb: mov rbp, rax ; 
2234ee: jmp 0x10411e ; 
2234f3: endbr64  ; 
2234f7: mov rbp, rax ; 
2234fa: jmp 0x10413e ; 
