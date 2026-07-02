/* Test conversions from signed integer types to float */
float int_to_float(int i) {
    return (float) i;
}

float long_to_float(long l) {
    return (float) l;
}
int main(void) {

    if (int_to_float(-100000) != -100000.0) {
        return 1;
    }

    // -16777217 is exactly halfway between -16777216 and -16777218;
    // ties-to-even rounding picks -16777216, which has the even significand
    if (long_to_float(-16777217l) != -16777216.0) {
        return 2;
    }

    // cast a constant to float to exercise rewrite rule for cvtsi2ss $const, dst
    float f = (float) 1073741825l; // 2^30 + 1; nearest float is 2^30
    if (f != 1073741824.0) {
        return 3;
    }

    return 0;
}
