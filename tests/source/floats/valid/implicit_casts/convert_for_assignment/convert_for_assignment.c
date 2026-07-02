#ifdef SUPPRESS_WARNINGS
#ifdef __clang__
#pragma GCC diagnostic ignored "-Wimplicit-const-int-float-conversion"
#pragma GCC diagnostic ignored "-Wliteral-conversion"
#endif
#endif
/* Test that we correctly perform conversion as if by assignment */

int check_args(long l, float d) { return l == 2 && d == -6.0; }

float return_float(void) {
    /* Implicitly convert this integer to the nearest float,
     * which is 18446744073709551616.0
     */
    return 18446744073709551586ul;
}

int check_assignment(float arg) {
    // arg = 4.9
    int i = 0;
    /* truncate arg to 4 */
    i = arg;
    return i == 4;
}

double float_to_double_assign(float f) {
    // f is implicitly converted to double
    double d = f;
    return d;
}

float double_to_float_assign(double d) {
    // d is implicitly converted to float
    float f = d;
    return f;
}

int main(void) {

    /* function arguments: 2.4 should be truncated to 2, -6 should be converted
     * to -6.0 */
    if (!check_args(2.4, -6)) {
        return 1;
    }

    /* return values */
    if (return_float() != 18446744073709551616.0) {
        return 2;
    }

    /* assignment statement */
    if (!check_assignment(4.9)) {
        return 3;
    }

    /* initializer */
    float d =
        18446744073709551586ul; // implicitly convert constant to nearest float

    if (d != 18446744073709551616.) {
        return 4;
    }

    /* implicit conversion from float to double in initializer */
    if (float_to_double_assign(100.0f) != 100.0) {
        return 5;
    }

    /* implicit conversion from double to float in initializer;
     * 16777217.0 (2^24 + 1) is represented exactly as a double, but is
     * exactly halfway between the two nearest floats, 16777216.0 and
     * 16777218.0; ties to even rounds this down to 16777216.0
     */
    if (double_to_float_assign(16777217.0) != 16777216.0f) {
        return 6;
    }

    return 0;
}
