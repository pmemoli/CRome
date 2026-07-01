/* Test addition, subtraction, multiplication, division, and negation with floats */

float point_one = 0.1;
float point_two = 0.2;
float point_three = 0.3;

float two = 2.0;
float three = 3.0;
float four = 4.0;
float twelveE30 = 12e30;

int addition(void) {
    return (point_one + point_two == 0.300000011920928955078125);
}

int subtraction(void) {
    return (four - 1.0 == 3.0);
}

int multiplication(void) {
    return (0.01f * point_three == 0.0030000000260770320892333984375);
}

int division(void) {
    return (7.0 / two == 3.5);
}

int negation(void) {
    float neg = -twelveE30;
    return !(12e30f + neg);
}

int complex_expression(void) {
    /* Test a more complex expression.
     * Note: all intermediate results in this expression
     * can be represented exactly, so we don't need to
     * consider the impact of rounding intermediate results.
     */

    float complex_expression = (two + three) - 127.5 * four;
    return complex_expression == -505.0;
}

int main(void) {

    if (!addition()) {
        return 1;
    }

    if (!subtraction()){
        return 2;
    }

    if (!multiplication()) {
        return 3;
    }

    if (!division()) {
        return 4;
    }

    if (!negation()) {
        return 5;
    }

    if (!complex_expression()) {
        return 6;
    }

    return 0;
}
