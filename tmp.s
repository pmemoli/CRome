    .globl return_static_variable
    .text
return_static_variable:
    pushq %rbp
    movq %rsp, %rbp
    subq $32,%rsp
    movss var.1(%rip),%xmm14
    movss %xmm14,-4(%rbp)
    cvtss2sd var.1(%rip),%xmm15
    movsd %xmm15,-12(%rbp)
    movsd -12(%rbp),%xmm14
    movsd %xmm14,-20(%rbp)
    movsd -20(%rbp),%xmm15
    addsd tmp.7(%rip),%xmm15
    movsd %xmm15,-20(%rbp)
    cvtsd2ss -20(%rbp),%xmm15
    movss %xmm15,-24(%rbp)
    movss -24(%rbp),%xmm14
    movss %xmm14,var.1(%rip)
    movss -4(%rbp),%xmm0
    movq %rbp, %rsp
    popq %rbp
    ret
    movl $0,%eax
    movq %rbp, %rsp
    popq %rbp
    ret
    .globl main
    .text
main:
    pushq %rbp
    movq %rsp, %rbp
    subq $32,%rsp
    call return_static_variable@PLT
    movss %xmm0,-4(%rbp)
    movss -4(%rbp),%xmm14
    movss %xmm14,-8(%rbp)
    cvtss2sd -8(%rbp),%xmm15
    movsd %xmm15,-16(%rbp)
    movsd -16(%rbp),%xmm15
    ucomisd tmp.8(%rip),%xmm15
    movl $0,-20(%rbp)
    setne -20(%rbp)
    movl -20(%rbp),%eax
    movq %rbp, %rsp
    popq %rbp
    ret
    movl $0,%eax
    movq %rbp, %rsp
    popq %rbp
    ret
    .data
    .align 4
var.1:
    .quad 4602678819172646912
    .section .rodata
    .align 8
tmp.7:
    .quad 4607182418800017408
    .section .rodata
    .align 8
tmp.8:
    .quad 4602678819172646912

    .section .note.GNU-stack,"",@progbits
