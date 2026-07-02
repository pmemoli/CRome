/* Test that we follow the calling convention for a float return type */
float d(void) {
    return 1234.e30;
}

int main(void) {
    float retval = d();
    return retval == 1233999985625344730829189036376064.0;
}
