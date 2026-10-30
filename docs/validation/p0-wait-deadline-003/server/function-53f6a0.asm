_ZNK12LongPollTask10Subscriber9IsTimeoutEv
53f6a0: endbr64
53f6a4: push rbx
53f6a5: mov rbx, rdi
53f6a8: xor edi, edi
53f6aa: call 0xefa60 ; PLT:time
53f6af: cmp rax, qword ptr [rbx]
53f6b2: pop rbx
53f6b3: setge al
53f6b6: ret
