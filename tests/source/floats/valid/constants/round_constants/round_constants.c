/* Make sure we round constant tokens to the nearest representable float */

int main(void) {
    /* This value is just above the midpoint between 1.0 and the next
     * representable float, 1.00000011920928955078125, so it should
     * round up to that value.
     */
    if (1.00000007f != 1.00000011920928955078125) {
        return 1;
    }

    /* Test round-to-even for a large constant token: 1073741888 lies
     * exactly halfway between 1073741824.0 and 1073741952.0. Since
     * 1073741824.0 has the even significand, ties-to-even rounding
     * picks it.
     */
    if (1073741888.0f != 1073741824.0) {
        return 2;
    }

    /* Another round-to-even case: 8388608.5 lies exactly halfway
     * between 8388608.0 (even significand) and 8388609.0 (odd
     * significand), so it should round down to 8388608.0.
     */
    if (8388608.5f != 8388608.0) {
        return 3;
    }

    /* And 8388609.5 lies exactly halfway between 8388609.0 (odd) and
     * 8388610.0 (even), so it should round up to 8388610.0.
     */
    if (8388609.5f != 8388610.0) {
        return 4;
    }

    return 0;
}
