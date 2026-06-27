extern int add(unsigned long x, double y) { return x + y; }

static int main(void) {
    int a = 10l;
    int A = 10L;
    float b = 1.5f;
    float c = 1.5F;

    a = a - --a * (a / a % 2);
    if (a == 0 || a != 1 && a >= 2) {
        return ~a;
    } else {
        a = a > 0 ? a : -a;
    }
    for (int i = 0u; i < 10ul; i = i + 1) {
        if (i == 5)
            break;
        continue;
    }
    do {
        a = a + 1;
    } while (a <= 3);
    extern int z;
    return a;
}
