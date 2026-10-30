734e0: push r15
734e2: push r14
734e4: mov r15d, ecx
734e7: push r13
734e9: push r12
734eb: mov r12, rdi
734ee: push rbp
734ef: push rbx
734f0: mov r13, rsi
734f3: mov r14, rdx
734f6: sub rsp, 0x48
734fa: mov rbx, rsp
734fd: lea rbp, [rsp + 0x20]
73502: mov rdi, rbx
73505: mov rax, qword ptr fs:[0x28]
7350e: mov qword ptr [rsp + 0x38], rax
73513: xor eax, eax
73515: call 0x227770 ; _ZN7PObjectC1Ev
7351a: mov rdi, rbp
7351d: call 0x227770 ; _ZN7PObjectC1Ev
73522: mov r9d, r15d
73525: mov r8, rbp
73528: mov rcx, r14
7352b: mov rdx, rbx
7352e: mov rsi, r13
73531: mov rdi, r12
73534: call 0x72030 ; _ZN14ProtocolClient7RequestERK7PObjectS2_RS0_S3_i
73539: mov rdi, rbp
7353c: mov r12d, eax
7353f: call 0x229be0 ; _ZN7PObjectD1Ev
73544: mov rdi, rbx
73547: call 0x229be0 ; _ZN7PObjectD1Ev
7354c: mov rdx, qword ptr [rsp + 0x38]
73551: xor rdx, qword ptr fs:[0x28]
7355a: mov eax, r12d
7355d: jne 0x7356e
7355f: add rsp, 0x48
73563: pop rbx
73564: pop rbp
73565: pop r12
73567: pop r13
73569: pop r14
7356b: pop r15
7356d: ret
7356e: call 0x1db00
73573: mov rdi, rbp
73576: mov r12, rax
73579: call 0x229be0 ; _ZN7PObjectD1Ev
7357e: mov rdi, rbx
73581: call 0x229be0 ; _ZN7PObjectD1Ev
73586: mov rdi, r12
73589: call 0x1e3a0
7358e: mov r12, rax
73591: jmp 0x7357e
