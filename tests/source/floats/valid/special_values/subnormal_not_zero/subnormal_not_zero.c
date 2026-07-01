int non_zero(float d) {
    return !d;
}

float multiply_by_large_num(float d) {
    return d * 2e20f;
}

int main(void) {

    /* Make sure subnormal numbers are not rounded to zero */
    float subnormal = 2.5e-40;

    /* Perform an operation on a subnormal number to produce a normal number */
    if (multiply_by_large_num(subnormal) != 5.0000011337973198085896857183196839713446024688892066478729248046875e-20) {
        return 2;
    }

    // subnormal is non-zero, so !subnormal should be zero
    return non_zero(subnormal);
}
