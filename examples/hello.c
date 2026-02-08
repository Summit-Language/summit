#include "freestanding.h"

uint8_t sm_std_io_read_u8(void);
const char* sm_std_io_readln(void);
void sm_std_io_print(const char* s);
void sm_std_io_print_i64(int64_t n);
void sm_std_io_println(const char* s);

int8_t main();

int8_t main() {
    sm_std_io_print("Enter your name: ");
    const const char* name = sm_std_io_readln();
    sm_std_io_print("Your name is: "); sm_std_io_print(name); sm_std_io_println("");
    sm_std_io_print("Enter your age: ");
    const uint8_t age = sm_std_io_read_u8();
    return 0;
}

void _start(void) {
    int8_t exit_code = main();
    syscall1(SYS_exit, exit_code);
}

