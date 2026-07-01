/* Test initializing static floats with integer constants and vice versa */

#ifdef SUPPRESS_WARNINGS
#ifdef __clang__
#pragma GCC diagnostic ignored "-Wimplicit-const-int-float-conversion"
#pragma GCC diagnostic ignored "-Wliteral-conversion"
#endif
#endif

// float variables

// can convert from small int/uint values without rounding, since they're
// well within the 24-bit range a float can represent exactly
float d1 = 8388607;
float d2 = 16777215u;

/* midway point between 33554430.0 and 33554432.0; ties to even rounds
 * this up to 33554432.0
 */
float d3 = 33554431l;

/* We'll round this down to 33554432.0 */
float d4 = 33554434l;

/* Using round-to-nearest, this rounds to 33554428.0 */
float d5 = 33554427ul;
float d6 = 33554434ul; // this is the same value as d4 and should round to the same float

/* This is exactly halfway between 33554428.0 and 33554432.0. We round
 * ties to even, so this rounds down to 33554428.0
 */
float d7 = 33554429ul;

float uninitialized; // should be initialized to 0.0

// integer variables

static int i = 4.9; // truncated to 4

int unsigned u = 42949.672923E1; // truncated to 429496u

// this token is first lexed as a float w/ value 33554432.0,
// then truncated down to long 33554432
long l = 33554431.f;

unsigned long ul = 18446742974197923840.f;

int main(void) {
    if (d1 != 8388607.) {
        return 1;
    }

    if (d2 != 16777215.) {
        return 2;
    }
    if (d3 != 33554432.) {
        return 3;
    }

    if (d4 != d3) {
        return 4;
    }

    if (d5 != 33554428.) {
        return 5;
    }

    if (d6 != d3) {
        return 6;
    }

    if (d7 != d5) {
        return 7;
    }

    if (uninitialized) {
        return 8;
    }

    if (i != 4) {
        return 9;
    }

    if (u != 429496u) {
        return 10;
    }

    if (l != 33554432l) {
        return 11;
    }

    if (ul != 18446742974197923840ul) {
        return 12;
    }

    return 0;
}
