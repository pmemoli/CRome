/* Test linking against a float defined in another file */
extern float d;

int main(void) {
    // 1e20 rounds to 100000002004087734272.0 when narrowed to float
    return d == 100000002004087734272.0;
}
