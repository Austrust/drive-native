_ZN13HandlerHelper34ConvertResponseErrorToHandlerErrorEj
address=0x11e600 size=70
11e600: cmp edi, 0x9004 ; 
11e606: je 0x11e640 ; 
11e608: lea eax, [rdi - 0x9003] ; 
11e60e: and eax, 0xfffffffd ; 
11e611: je 0x11e630 ; 
11e613: cmp edi, 0x8004 ; 
11e619: je 0x11e620 ; 
11e61b: jmp 0x22c090 ; 
11e620: mov eax, 0xffffffcc ; 
11e625: ret  ; 
11e626: nop word ptr cs:[rax + rax] ; 
11e630: mov eax, 0xffffffe0 ; 
11e635: ret  ; 
11e636: nop word ptr cs:[rax + rax] ; 
11e640: mov eax, 0xffffffe5 ; 
11e645: ret  ; 
