    .globl main
    .text
main:
    pushq %rbp
    movq %rsp, %rbp
    subq $112,%rsp
    movl $2185232384,-4(%rbp)
    movq $144115196665790464,%r10
    movq %r10,-12(%rbp)
    movsd tmp.13(%rip),%xmm14
    movsd %xmm14,-20(%rbp)
    leaq -4(%rbp),%r11
    movq %r11,-28(%rbp)
    movq -28(%rbp),%r10
    movq %r10,-36(%rbp)
    leaq -12(%rbp),%r11
    movq %r11,-44(%rbp)
    movq -44(%rbp),%r10
    movq %r10,-52(%rbp)
    leaq -20(%rbp),%r11
    movq %r11,-60(%rbp)
    movq -60(%rbp),%r10
    movq %r10,-68(%rbp)
    movl $10,-72(%rbp)
    movq -36(%rbp),%rax
    movl -72(%rbp),%r10d
    movl %r10d,(%rax)
    movl $20,-76(%rbp)
    negl -76(%rbp)
    movl -76(%rbp),%r10d
    movslq %r10d,%r11
    movq %r11,-84(%rbp)
    movq -52(%rbp),%rax
    movq -84(%rbp),%r10
    movq %r10,(%rax)
    movq -68(%rbp),%rax
    movsd tmp.14(%rip),%xmm14
    movsd %xmm14,(%rax)
    movl $10,-88(%rbp)
    movl -88(%rbp),%r10d
    cmpl %r10d,-4(%rbp)
    movl $0,-92(%rbp)
    setne -92(%rbp)
    cmpl $0,-92(%rbp)
    je .Lend.1
    movl $1,%eax
    movq %rbp, %rsp
    popq %rbp
    ret
    .Lend.1:
    movl $20,-96(%rbp)
    negl -96(%rbp)
    movl -96(%rbp),%r10d
    movslq %r10d,%r11
    movq %r11,-104(%rbp)
    movq -104(%rbp),%r10
    cmpq %r10,-12(%rbp)
    movl $0,-108(%rbp)
    setne -108(%rbp)
    cmpl $0,-108(%rbp)
    je .Lend.2
    movl $2,%eax
    movq %rbp, %rsp
    popq %rbp
    ret
    .Lend.2:
    movsd -20(%rbp),%xmm15
    comisd tmp.14(%rip),%xmm15
    movl $0,-112(%rbp)
    setne -112(%rbp)
    cmpl $0,-112(%rbp)
    je .Lend.3
    movl $3,%eax
    movq %rbp, %rsp
    popq %rbp
    ret
    .Lend.3:
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
tmp.13:
    .quad 5355091182177117338
    .section .rodata
    .align 8
tmp.14:
    .quad 4629165614481119642

    .section .note.GNU-stack,"",@progbits
