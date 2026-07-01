float fun(float x) {
    if (x > 2)
        return x;
    else {
        float ret = fun(x + 2); // ret = 3.0
        return ret + x; // return 4.0
    }
}
