3191b0: push rbx
3191b1: sub rsp, 0x20
3191b5: mov edx, dword ptr [rdi + 0x18]
3191b8: mov rax, qword ptr fs:[0x28]
3191c1: mov qword ptr [rsp + 0x18], rax
3191c6: xor eax, eax
3191c8: test edx, edx
3191ca: je 0x319207
3191cc: mov rbx, rdi
3191cf: lea rdi, [rsp + 0x10]
3191d4: mov qword ptr [rsp + 8], rsi
3191d9: call 0x1e5e0
3191de: mov eax, dword ptr [rbx + 0x18]
3191e1: pxor xmm1, xmm1
3191e5: mov rsi, qword ptr [rsp + 8]
3191ea: pxor xmm0, xmm0
3191ee: cvtsi2sd xmm1, rax
3191f3: mov rax, qword ptr [rsp + 0x10]
3191f8: sub rax, rsi
3191fb: cvtsi2sd xmm0, rax
319200: ucomisd xmm0, xmm1
319204: seta al
319207: mov rdx, qword ptr [rsp + 0x18]
31920c: xor rdx, qword ptr fs:[0x28]
319215: jne 0x31921d
319217: add rsp, 0x20
31921b: pop rbx
31921c: ret
31921d: call 0x1db00
