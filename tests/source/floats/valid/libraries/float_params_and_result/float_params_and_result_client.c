float get_max(float a, float b, float c, float d,
               float e, float f, float g, float h,
               float i, float j, float k);

int main(void)
{
    float result = get_max(100.3, 200.1, 0.01, 1.00004e5, 55.555, -4., 6543.2,
                            9e9, 8e8, 7.6,  10e3 * 11e5);
    // 10e3 * 11e5 = 11000000000, which rounds to the nearest float,
    // 11000000512.0, when passed as an argument
    return result == 11000000512.0;
}
