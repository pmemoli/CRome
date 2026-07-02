/* Test conversions from float to the double type */

int main(void) {

    // float to double should always remain equal
    double d = (double)100.0f;
    if (d != 100.0) {
        return 1;
    }

    return 0;
}
