#include "io.h"
#include "string_utils.h"

static long sm_write(int fd, const void* buf, size_t count) {
    return syscall3(SYS_write, fd, (long)buf, count);
}

void sm_std_io_print(const char* s) {
    if (s == NULL) return;
    size_t len = sm_strlen(s);
    sm_write(STDOUT_FILENO, s, len);
}

void sm_std_io_println(const char* s) {
    if (s != NULL) {
        size_t len = sm_strlen(s);
        sm_write(STDOUT_FILENO, s, len);
    }
    sm_write(STDOUT_FILENO, "\n", 1);
}

void sm_std_io_eprint(const char* s) {
    if (s == NULL) return;
    size_t len = sm_strlen(s);
    sm_write(STDERR_FILENO, s, len);
}

void sm_std_io_eprintln(const char* s) {
    if (s != NULL) {
        size_t len = sm_strlen(s);
        sm_write(STDERR_FILENO, s, len);
    }
    sm_write(STDERR_FILENO, "\n", 1);
}

void sm_std_io_print_i64(int64_t n) {
    char buf[32];
    sm_int_to_string(n, buf);
    sm_std_io_print(buf);
}

void sm_std_io_println_i64(int64_t n) {
    char buf[32];
    sm_int_to_string(n, buf);
    sm_std_io_println(buf);
}

void sm_std_io_print_u64(uint64_t n) {
    char buf[32];
    sm_uint64_to_string(n, buf);
    sm_std_io_print(buf);
}

void sm_std_io_println_u64(uint64_t n) {
    char buf[32];
    sm_uint64_to_string(n, buf);
    sm_std_io_println(buf);
}

void sm_std_io_print_i128(int128_t n) {
    char buf[64];
    sm_int128_to_string(n, buf);
    sm_std_io_print(buf);
}

void sm_std_io_println_i128(int128_t n) {
    char buf[64];
    sm_int128_to_string(n, buf);
    sm_std_io_println(buf);
}

void sm_std_io_print_u128(uint128_t n) {
    char buf[64];
    sm_uint128_to_string(n, buf);
    sm_std_io_print(buf);
}

void sm_std_io_println_u128(uint128_t n) {
    char buf[64];
    sm_uint128_to_string(n, buf);
    sm_std_io_println(buf);
}

void sm_std_io_print_bool(bool b) {
    if (b) {
        sm_std_io_print("true");
    } else {
        sm_std_io_print("false");
    }
}

void sm_std_io_println_bool(bool b) {
    if (b) {
        sm_std_io_println("true");
    } else {
        sm_std_io_println("false");
    }
}

const char* sm_std_io_readln(void) {
    static char buffer[4096];
    size_t i = 0;

    while (i < sizeof(buffer) - 1) {
        char c;
        long result = syscall3(SYS_read, STDIN_FILENO, (long)&c, 1);

        if (result <= 0) {
            break;
        }

        if (c == '\n') {
            break;
        }

        buffer[i++] = c;
    }

    buffer[i] = '\0';
    return buffer;
}