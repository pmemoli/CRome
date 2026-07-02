/* Test conversions from float to the signed integer types */

int float_to_int(float d) {
    return (int) d;
}

long float_to_long(float d) {
    return (long) d;
}

int main(void) {

    // a float this large has no fractional precision left, but it's still
    // too big for an int, so it must be truncated into a long
    long l = float_to_long(3000000000.0);
    if (l != 3000000000l) {
        return 1;
    }

    int i = float_to_int(-200000.75);
    // negative value should be truncated towards 0
    if (i != -200000) {
        return 2;
    }

    return 0;
}
