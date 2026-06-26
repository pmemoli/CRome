    .globl check_args
    .text
check_args:
    pushq %rbp
    movq %rsp, %rbp
    subq $48,%rsp
    movq %rdi,-8(%rbp)
    movsd %xmm0,-16(%rbp)
    movl $2,%r10d
    movslq %r10d,%r11
    movq %r11,-24(%rbp)
    movq -24(%rbp),%r10
    cmpq %r10,-8(%rbp)
    movl $0,-28(%rbp)
    sete -28(%rbp)
    cmpl $0,-28(%rbp)
    je .Lfalse_result.1
    movsd tmp.20(%rip),%xmm14
    movsd %xmm14,-36(%rbp)
    movsd -36(%rbp),%xmm15
    xorpd tmp.21(%rip),%xmm15
    movsd %xmm15,-36(%rbp)
    movsd -16(%rbp),%xmm15
    comisd -36(%rbp),%xmm15
    movl $0,-40(%rbp)
    sete -40(%rbp)
    cmpl $0,-40(%rbp)
    je .Lfalse_result.1
    .Ltrue_result.1:
    movl $1,-44(%rbp)
    jmp .Lend.1
    .Lfalse_result.1:
    movl $0,-44(%rbp)
    .Lend.1:
    movl -44(%rbp),%eax
    movq %rbp, %rsp
    popq %rbp
    ret
    movl $0,%eax
    movq %rbp, %rsp
    popq %rbp
    ret
    .globl return_double
    .text
return_double:
    pushq %rbp
    movq %rsp, %rbp
    subq $16,%rsp
    movq $18446744073709551586,%r10
    movq %r10,-8(%rbp)
    movsd -8(%rbp),%xmm0
    movq %rbp, %rsp
    popq %rbp
    ret
    movl $0,%eax
    movq %rbp, %rsp
    popq %rbp
    ret
    .globl check_assignment
    .text
check_assignment:
    pushq %rbp
    movq %rsp, %rbp
    subq $32,%rsp
    movsd %xmm0,-8(%rbp)
    movl $0,-12(%rbp)
    cvttsd2sil -8(%rbp),%r11d
    movl %r11d,-16(%rbp)
    movl -16(%rbp),%r10d
    movl %r10d,-12(%rbp)
    cmpl $4,-12(%rbp)
    movl $0,-20(%rbp)
    sete -20(%rbp)
    movl -20(%rbp),%eax
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
    subq $80,%rsp
    movsd tmp.22(%rip),%xmm14
    movsd %xmm14,-8(%rbp)
    movl $6,-12(%rbp)
    negl -12(%rbp)
    cvtsi2sdl -12(%rbp),%xmm15
    movsd %xmm15,-20(%rbp)
    movq -8(%rbp),%rdi
    movsd -20(%rbp),%xmm0
    call check_args@PLT
    movl %eax,-24(%rbp)
    cmpl $0,-24(%rbp)
    movl $0,-28(%rbp)
    sete -28(%rbp)
    cmpl $0,-28(%rbp)
    je .Lend.2
    movl $1,%eax
    movq %rbp, %rsp
    popq %rbp
    ret
    .Lend.2:
    call return_double@PLT
    movsd %xmm0,-36(%rbp)
    movsd -36(%rbp),%xmm15
    comisd tmp.23(%rip),%xmm15
    movl $0,-40(%rbp)
    setne -40(%rbp)
    cmpl $0,-40(%rbp)
    je .Lend.3
    movl $2,%eax
    movq %rbp, %rsp
    popq %rbp
    ret
    .Lend.3:
    movsd tmp.24(%rip),%xmm0
    call check_assignment@PLT
    movl %eax,-44(%rbp)
    cmpl $0,-44(%rbp)
    movl $0,-48(%rbp)
    sete -48(%rbp)
    cmpl $0,-48(%rbp)
    je .Lend.4
    movl $3,%eax
    movq %rbp, %rsp
    popq %rbp
    ret
    .Lend.4:
    movq $18446744073709551586,%r10
    movq %r10,-56(%rbp)
    movsd -56(%rbp),%xmm14
    movsd %xmm14,-64(%rbp)
    movsd -64(%rbp),%xmm15
    comisd tmp.23(%rip),%xmm15
    movl $0,-68(%rbp)
    setne -68(%rbp)
    cmpl $0,-68(%rbp)
    je .Lend.5
    movl $4,%eax
    movq %rbp, %rsp
    popq %rbp
    ret
    .Lend.5:
    movl $0,%eax
    movq %rbp, %rsp
    popq %rbp
    ret
    movl $0,%eax
    movq %rbp, %rsp
    popq %rbp
    ret
    .section .rodata
    .align 8
tmp.24:
    .quad 4617202927970916762
    .section .rodata
    .align 8
tmp.22:
    .quad 4612586738352862003
    .section .rodata
    .align 8
tmp.20:
    .quad 4618441417868443648
    .section .rodata
    .align 16
tmp.21:
    .quad 9223372036854775808
    .section .rodata
    .align 8
tmp.23:
    .quad 4895412794951729152

    .section .note.GNU-stack,"",@progbits
