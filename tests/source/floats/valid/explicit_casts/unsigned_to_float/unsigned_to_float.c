/* Test conversions from unsigned integer types to floats */
float uint_to_float(unsigned int ui) {
    return (float) ui;
}

float ulong_to_float(unsigned long ul) {
    return (float) ul;
}

int main(void) {

    // uint that's smaller than INT_MAX
    if (uint_to_float(1000u) != 1000.0) {
        return 1;
    }

    // uint that's larger than INT_MAX, so we can't just use cvtsi2ss
    if (uint_to_float(4294967200u) != 4294967296.0) {
        return 2;
    }

    // ulong that's smaller than LONG_MAX
    if (ulong_to_float(138512825844ul) != 138512826368.0) {
        return 3;
    }

    // ulong that's larger than LONG_MAX
    if (ulong_to_float(10223372036854775816ul) != 10223372364758908928.0) {
        return 4;
    }

    /* To test our rounding behavior, use values that straddle a
     * representable-float boundary near 2^40
     */

    /* This value is exactly between 1099511562240.0 and 1099511627776.0
     * Using ties-to-even rounding, we'll round it up to
     * 1099511627776.0, which has an even significand
     */
    if (ulong_to_float(1099511595008ul) != 1099511627776.0) {
        return 5;
    }

    /* This value is closer to 1099511627776.0 than 1099511562240.0,
     * so we should round up.
     */
    if (ulong_to_float(1099511595009ul) != 1099511627776.0) {
        return 6;
    }

    /* This value is closer to 1099511562240.0 than 1099511627776.0,
     * so round down */
    if (ulong_to_float(1099511595007ul) != 1099511562240.0) {
        return 7;
    }

    return 0;
}
