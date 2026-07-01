/* Make sure we can call floating-point functions from the standard library */

/* We need to declare these functions ourselves since we can't #include <math.h> */

// fused multiply and add: (x * y) + z
// note: only the final result of the whole calculation is rounded,
// not the intermediate result x * y
float fmaf(float x, float y, float z);

float ldexpf(float x, int exp); // x * 2^exp

int main(void) {
    /* fmaf must round the exact mathematical result of x * y + z only once;
     * if x * y were rounded to a float first, the z term would be lost
     * entirely and we'd get a different (wrong) answer
     */
    float fma_result = fmaf(3.0f, 1e13f, 1.0f);
    float ldexp_result = ldexpf(1.4325f, 10);
    if (fma_result != 30000000532480.0) {
        return 1;
    }

    if (ldexp_result != 1466.8800048828125) {
        return 2;
    }

    return 0;
}
