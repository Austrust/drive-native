_ZN12LongPollTask10SubscriberC1ER10Connectioni
53f460: endbr64
53f464: push rbp
53f465: lea rax, [rdi + 0x18]
53f469: movsxd rbp, edx
53f46c: push rbx
53f46d: mov rbx, rdi
53f470: sub rsp, 8
53f474: mov qword ptr [rdi + 0x28], rax
53f478: mov qword ptr [rdi + 0x30], rax
53f47c: lea rax, [rdi + 0x50]
53f480: mov qword ptr [rdi + 0x40], rax
53f484: lea rax, [rdi + 0x70]
53f488: mov dword ptr [rdi + 0x18], 0
53f48f: mov qword ptr [rdi + 0x20], 0
53f497: mov qword ptr [rdi + 0x38], 0
53f49f: mov qword ptr [rdi + 0x48], 0
53f4a7: mov byte ptr [rdi + 0x50], 0
53f4ab: mov qword ptr [rdi + 0x60], rax
53f4af: mov qword ptr [rdi + 0x68], 0
53f4b7: mov byte ptr [rdi + 0x70], 0
53f4bb: mov qword ptr [rdi + 8], rsi
53f4bf: xor edi, edi
53f4c1: call 0xefa60 ; PLT:time
53f4c6: add rbp, rax
53f4c9: mov qword ptr [rbx], rbp
53f4cc: add rsp, 8
53f4d0: pop rbx
53f4d1: pop rbp
53f4d2: ret
