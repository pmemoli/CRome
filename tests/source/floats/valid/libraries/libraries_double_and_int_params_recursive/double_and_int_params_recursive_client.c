/* This test case is identical to chapter13/valid/function_calls/double_and_int_params_recursive.c
 * but split across two files */
int fun(int i1, float d1, int i2, float d2, int i3, float d3,
        int i4, float d4, int i5, float d5, int i6, float d6,
        int i7, float d7, int i8, float d8, int i9, float d9);
int main(void) {
    float d = fun(1, 2.0, 3, 4.0, 5, 6.0, 7, 8.0, 9, 10.0, 11, 12.0, 13, 14.0, 15, 16.0, 17, 18.0);
    return (d == 78.00);
}
