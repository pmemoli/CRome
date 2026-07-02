/* Test conversions from double to the float type */

int main(void) {

    // 100 can be represented as both a float and a double, so they should
    // compare equal
    float d = (float)100.0;
    if (d != 100.0f) {
        return 1;
    }

    // 16777217.0 (2^24 + 1) is represented exactly as a double, but is
    // exactly halfway between the two nearest floats, 16777216.0 and
    // 16777218.0; ties to even rounds this down to 16777216.0
    d = (float)16777217.0;
    if (d != 16777216.0f) {
        return 2;
    }

    return 0;
}
