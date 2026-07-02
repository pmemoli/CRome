/* Test that we've properly implemented the calling convention
 * for passing floats and ints in registers
 */
int check_arguments(float d1, float d2, int i1, float d3, float d4, int i2, int i3,
                    int i4, float d5, float d6, float d7, int i5, float d8) {

    if (d1 != 1.0) {
        return 1;
    }
    if (d2 != 2.0) {
        return 2;
    }
    if (d3 != 3.0) {
        return 3;
    }
    if (d4 != 4.0 ){
        return 4;
    }
    if (d5 != 5.0){
        return 5;
    }

    if (d6 != 6.0 ){
        return 6;
    }
    if (d7 != 7.0 ){
        return 7;
    }
    if (d8 != 8.0 ){
        return 8;
    }
    if (i1 != 101 ){
        return 9;
    }
    if (i2 != 102 ){
        return 10;
    }
    if (i3 != 103){
        return 11;
    }
    if (i4 != 104) {
        return 12;
    }
    if (i5 != 105) {
        return 13;
    }
    return 0;
}

int main(void) {
    return check_arguments(1.0, 2.0, 101, 3.0, 4.0, 102, 103, 104, 5.0, 6.0, 7.0, 105, 8.0);
}
