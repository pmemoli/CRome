// Test reading and writing a local static float

// Return old value, then increment by one
float return_static_variable(void) {
    static float d = 0.5;
    float ret = d;
    d = d + 1.0;
    return ret;
}

int main(void) {
    float d1 = return_static_variable();
    float d2 = return_static_variable();
    float d3 = return_static_variable();
    if (d1 != 0.5) {
        return 1;
    }
    if (d2 != 1.5) {
        return 2;
    }
    if (d3 != 2.5) {
        return 3;
    }
    return 0;
}
