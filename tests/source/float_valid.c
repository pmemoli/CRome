static float bar = 1.0;

float xd(float foo) {
    double god = 1.0f;
    return foo + god + bar;
}

int main(void) {
    double foo = xd(1.0f);
    return foo;
}
