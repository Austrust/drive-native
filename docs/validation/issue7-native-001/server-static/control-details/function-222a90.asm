_ZN14DefaultACLTest7GetPathB5cxx11Ev
222a90: endbr64  ; 
222a94: push rbp ; 
222a95: push rbx ; 
222a96: mov rbx, rdi ; 
222a99: sub rsp, 0x38 ; 
222a9d: mov rax, qword ptr fs:[0x28] ; 
222aa6: mov qword ptr [rsp + 0x28], rax ; 
222aab: mov rax, qword ptr [rsi + 8] ; 
222aaf: lea rbp, [rsp + 0x10] ; 
222ab4: mov rdi, qword ptr [rax] ; 
222ab7: call 0x550d30 ; _ZN7Request9GetHeaderEv
222abc: mov rsi, rsp ; 
222abf: mov qword ptr [rsp], rbp ; 
222ac3: mov rdi, rax ; 
222ac6: mov dword ptr [rsp + 0x10], 0x68746170 ; 
222ace: mov qword ptr [rsp + 8], 4 ; 
222ad7: mov byte ptr [rsp + 0x14], 0 ; 
222adc: call 0xf1030 ; PLT:_ZNR7PObjectixERKNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEE
222ae1: mov rsi, rax ; 
222ae4: mov rdi, rbx ; 
222ae7: call 0xeeef0 ; PLT:_ZNKR7PObject8asStringB5cxx11Ev
222aec: mov rdi, qword ptr [rsp] ; 
222af0: cmp rdi, rbp ; 
222af3: je 0x222b03 ; 
222af5: mov rax, qword ptr [rsp + 0x10] ; 
222afa: lea rsi, [rax + 1] ; 
222afe: call 0xea4a0 ; PLT:_ZdlPvm
222b03: mov rax, qword ptr [rsp + 0x28] ; 
222b08: sub rax, qword ptr fs:[0x28] ; 
222b11: jne 0x222b1d ; 
222b13: add rsp, 0x38 ; 
222b17: mov rax, rbx ; 
222b1a: pop rbx ; 
222b1b: pop rbp ; 
222b1c: ret  ; 
222b1d: call 0xead00 ; PLT:__stack_chk_fail
222b22: endbr64  ; 
222b26: mov rbx, rax ; 
222b29: jmp 0x10403e ; 
