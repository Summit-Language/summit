#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

// System call numbers for x86_64 Linux
#define SYS_WRITE 1
#define SYS_READ 0
#define SYS_EXIT 60

// File descriptors
#define STDOUT_FILENO 1
#define STDERR_FILENO 2

// Raw syscall interface
static inline long syscall1(long number, long arg1) {
    long ret;
    __asm__ volatile (
        "syscall"
        : "=a"(ret)
        : "a"(number), "D"(arg1)
        : "rcx", "r11", "memory"
    );
    return ret;
}

static inline long syscall3(long number, long arg1, long arg2, long arg3) {
    long ret;
    __asm__ volatile (
        "syscall"
        : "=a"(ret)
        : "a"(number), "D"(arg1), "S"(arg2), "d"(arg3)
        : "rcx", "r11", "memory"
    );
    return ret;
}

// String length function
static size_t sm_strlen(const char* s) {
    size_t len = 0;
    while (s[len]) len++;
    return len;
}

// Write to file descriptor using raw syscall
static long sm_write(int fd, const void* buf, size_t count) {
    return syscall3(SYS_WRITE, fd, (long)buf, count);
}

// Print string to stdout
void sm_std_io_print(const char* s) {
    if (s == NULL) return;
    size_t len = sm_strlen(s);
    sm_write(STDOUT_FILENO, s, len);
}

// Print string to stdout with newline
void sm_std_io_println(const char* s) {
    if (s != NULL) {
        size_t len = sm_strlen(s);
        sm_write(STDOUT_FILENO, s, len);
    }
    sm_write(STDOUT_FILENO, "\n", 1);
}

// Print to stderr
void sm_std_io_eprint(const char* s) {
    if (s == NULL) return;
    size_t len = sm_strlen(s);
    sm_write(STDERR_FILENO, s, len);
}

// Print to stderr with newline
void sm_std_io_eprintln(const char* s) {
    if (s != NULL) {
        size_t len = sm_strlen(s);
        sm_write(STDERR_FILENO, s, len);
    }
    sm_write(STDERR_FILENO, "\n", 1);
}

// Convert integer to string
static void sm_int_to_string(int64_t num, char* buf) {
    int i = 0;
    int is_negative = 0;
    uint64_t unsigned_num;

    if (num == 0) {
        buf[i++] = '0';
        buf[i] = '\0';
        return;
    }

    if (num < 0) {
        is_negative = 1;
        unsigned_num = (uint64_t)(-(num + 1)) + 1;
    } else {
        unsigned_num = (uint64_t)num;
    }

    char temp[32];
    int j = 0;
    while (unsigned_num > 0) {
        temp[j++] = '0' + (unsigned_num % 10);
        unsigned_num /= 10;
    }

    if (is_negative) {
        buf[i++] = '-';
    }

    while (j > 0) {
        buf[i++] = temp[--j];
    }
    buf[i] = '\0';
}

// Convert unsigned 64-bit integer to string
static void sm_uint64_to_string(uint64_t num, char* buf) {
    int i = 0;

    if (num == 0) {
        buf[i++] = '0';
        buf[i] = '\0';
        return;
    }

    char temp[32];
    int j = 0;
    while (num > 0) {
        temp[j++] = '0' + (num % 10);
        num /= 10;
    }

    while (j > 0) {
        buf[i++] = temp[--j];
    }
    buf[i] = '\0';
}

// Convert 128-bit integer to string
static void sm_int128_to_string(__int128 num, char* buf) {
    int i = 0;
    int is_negative = 0;
    unsigned __int128 unsigned_num;

    if (num == 0) {
        buf[i++] = '0';
        buf[i] = '\0';
        return;
    }

    if (num < 0) {
        is_negative = 1;
        unsigned_num = (unsigned __int128)(-(num + 1)) + 1;
    } else {
        unsigned_num = (unsigned __int128)num;
    }

    char temp[64];
    int j = 0;
    while (unsigned_num > 0) {
        temp[j++] = '0' + (unsigned_num % 10);
        unsigned_num /= 10;
    }

    if (is_negative) {
        buf[i++] = '-';
    }

    while (j > 0) {
        buf[i++] = temp[--j];
    }
    buf[i] = '\0';
}

// Convert unsigned 128-bit integer to string
static void sm_uint128_to_string(unsigned __int128 num, char* buf) {
    int i = 0;

    if (num == 0) {
        buf[i++] = '0';
        buf[i] = '\0';
        return;
    }

    char temp[64];
    int j = 0;
    while (num > 0) {
        temp[j++] = '0' + (num % 10);
        num /= 10;
    }

    while (j > 0) {
        buf[i++] = temp[--j];
    }
    buf[i] = '\0';
}

// Print integer to stdout
void sm_std_io_print_i64(int64_t n) {
    char buf[32];
    sm_int_to_string(n, buf);
    sm_std_io_print(buf);
}

// Print integer to stdout with newline
void sm_std_io_println_i64(int64_t n) {
    char buf[32];
    sm_int_to_string(n, buf);
    sm_std_io_println(buf);
}

// Print unsigned 64-bit integer to stdout
void sm_std_io_print_u64(uint64_t n) {
    char buf[32];
    sm_uint64_to_string(n, buf);
    sm_std_io_print(buf);
}

// Print unsigned 64-bit integer to stdout with newline
void sm_std_io_println_u64(uint64_t n) {
    char buf[32];
    sm_uint64_to_string(n, buf);
    sm_std_io_println(buf);
}

// Print 128-bit integer to stdout
void sm_std_io_print_i128(__int128 n) {
    char buf[64];
    sm_int128_to_string(n, buf);
    sm_std_io_print(buf);
}

// Print 128-bit integer to stdout with newline
void sm_std_io_println_i128(__int128 n) {
    char buf[64];
    sm_int128_to_string(n, buf);
    sm_std_io_println(buf);
}

// Print unsigned 128-bit integer to stdout
void sm_std_io_print_u128(unsigned __int128 n) {
    char buf[64];
    sm_uint128_to_string(n, buf);
    sm_std_io_print(buf);
}

// Print unsigned 128-bit integer to stdout with newline
void sm_std_io_println_u128(unsigned __int128 n) {
    char buf[64];
    sm_uint128_to_string(n, buf);
    sm_std_io_println(buf);
}

// Print boolean to stdout
void sm_std_io_print_bool(bool b) {
    if (b) {
        sm_std_io_print("true");
    } else {
        sm_std_io_print("false");
    }
}

// Print boolean to stdout with newline
void sm_std_io_println_bool(bool b) {
    if (b) {
        sm_std_io_println("true");
    } else {
        sm_std_io_println("false");
    }
}

// Read a line from stdin
const char* sm_std_io_readln(void) {
    static char buffer[4096];
    size_t i = 0;

    while (i < sizeof(buffer) - 1) {
        char c;
        long result = syscall3(SYS_READ, 0, (long)&c, 1);

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

// Helper function to parse integer from string
static int sm_parse_i8(const char* str, int8_t* out) {
    long val = 0;
    int sign = 1;
    const char* p = str;

    while (*p == ' ' || *p == '\t') p++;

    if (*p == '-') {
        sign = -1;
        p++;
    } else if (*p == '+') {
        p++;
    }

    if (*p < '0' || *p > '9') return 0;

    while (*p >= '0' && *p <= '9') {
        val = val * 10 + (*p - '0');
        p++;
    }

    val *= sign;

    if (val < -128 || val > 127) return 0;

    *out = (int8_t)val;
    return 1;
}

static int sm_parse_u8(const char* str, uint8_t* out) {
    unsigned long val = 0;
    const char* p = str;

    while (*p == ' ' || *p == '\t') p++;
    if (*p == '-') return 0;
    if (*p == '+') p++;
    if (*p < '0' || *p > '9') return 0;

    while (*p >= '0' && *p <= '9') {
        val = val * 10 + (*p - '0');
        p++;
    }

    if (val > 255) return 0;
    *out = (uint8_t)val;
    return 1;
}

static int sm_parse_i16(const char* str, int16_t* out) {
    long val = 0;
    int sign = 1;
    const char* p = str;

    while (*p == ' ' || *p == '\t') p++;
    if (*p == '-') { sign = -1; p++; } else if (*p == '+') p++;
    if (*p < '0' || *p > '9') return 0;

    while (*p >= '0' && *p <= '9') {
        val = val * 10 + (*p - '0');
        p++;
    }

    val *= sign;
    if (val < -32768 || val > 32767) return 0;
    *out = (int16_t)val;
    return 1;
}

static int sm_parse_u16(const char* str, uint16_t* out) {
    unsigned long val = 0;
    const char* p = str;

    while (*p == ' ' || *p == '\t') p++;
    if (*p == '-') return 0;
    if (*p == '+') p++;
    if (*p < '0' || *p > '9') return 0;

    while (*p >= '0' && *p <= '9') {
        val = val * 10 + (*p - '0');
        p++;
    }

    if (val > 65535) return 0;
    *out = (uint16_t)val;
    return 1;
}

static int sm_parse_i32(const char* str, int32_t* out) {
    long long val = 0;
    int sign = 1;
    const char* p = str;

    while (*p == ' ' || *p == '\t') p++;
    if (*p == '-') { sign = -1; p++; } else if (*p == '+') p++;
    if (*p < '0' || *p > '9') return 0;

    while (*p >= '0' && *p <= '9') {
        val = val * 10 + (*p - '0');
        p++;
    }

    val *= sign;
    if (val < -2147483648LL || val > 2147483647LL) return 0;
    *out = (int32_t)val;
    return 1;
}

static int sm_parse_u32(const char* str, uint32_t* out) {
    unsigned long long val = 0;
    const char* p = str;

    while (*p == ' ' || *p == '\t') p++;
    if (*p == '-') return 0;
    if (*p == '+') p++;
    if (*p < '0' || *p > '9') return 0;

    while (*p >= '0' && *p <= '9') {
        val = val * 10 + (*p - '0');
        p++;
    }

    if (val > 4294967295ULL) return 0;
    *out = (uint32_t)val;
    return 1;
}

static int sm_parse_i64(const char* str, int64_t* out) {
    int64_t val = 0;
    int sign = 1;
    const char* p = str;

    while (*p == ' ' || *p == '\t') p++;
    if (*p == '-') { sign = -1; p++; } else if (*p == '+') p++;
    if (*p < '0' || *p > '9') return 0;

    while (*p >= '0' && *p <= '9') {
        int64_t digit = *p - '0';
        if (val > (9223372036854775807LL - digit) / 10) return 0;
        val = val * 10 + digit;
        p++;
    }

    if (sign == -1) {
        if (val > 9223372036854775807LL) return 0;
        val = -val;
    }

    *out = val;
    return 1;
}

static int sm_parse_u64(const char* str, uint64_t* out) {
    uint64_t val = 0;
    const char* p = str;

    while (*p == ' ' || *p == '\t') p++;
    if (*p == '-') return 0;
    if (*p == '+') p++;
    if (*p < '0' || *p > '9') return 0;

    while (*p >= '0' && *p <= '9') {
        uint64_t digit = *p - '0';
        if (val > (18446744073709551615ULL - digit) / 10) return 0;
        val = val * 10 + digit;
        p++;
    }

    *out = val;
    return 1;
}

// Read functions that call readln and parse
int8_t sm_std_io_read_i8(void) {
    const char* input = sm_std_io_readln();
    int8_t result;
    if (!sm_parse_i8(input, &result)) {
        sm_std_io_eprintln("Error: Invalid i8 input");
        syscall1(60, 1);
        __builtin_unreachable();
    }
    return result;
}

uint8_t sm_std_io_read_u8(void) {
    const char* input = sm_std_io_readln();
    uint8_t result;
    if (!sm_parse_u8(input, &result)) {
        sm_std_io_eprintln("Error: Invalid u8 input");
        syscall1(60, 1);
        __builtin_unreachable();
    }
    return result;
}

int16_t sm_std_io_read_i16(void) {
    const char* input = sm_std_io_readln();
    int16_t result;
    if (!sm_parse_i16(input, &result)) {
        sm_std_io_eprintln("Error: Invalid i16 input");
        syscall1(60, 1);
        __builtin_unreachable();
    }
    return result;
}

uint16_t sm_std_io_read_u16(void) {
    const char* input = sm_std_io_readln();
    uint16_t result;
    if (!sm_parse_u16(input, &result)) {
        sm_std_io_eprintln("Error: Invalid u16 input");
        syscall1(60, 1);
        __builtin_unreachable();
    }
    return result;
}

int32_t sm_std_io_read_i32(void) {
    const char* input = sm_std_io_readln();
    int32_t result;
    if (!sm_parse_i32(input, &result)) {
        sm_std_io_eprintln("Error: Invalid i32 input");
        syscall1(60, 1);
        __builtin_unreachable();
    }
    return result;
}

uint32_t sm_std_io_read_u32(void) {
    const char* input = sm_std_io_readln();
    uint32_t result;
    if (!sm_parse_u32(input, &result)) {
        sm_std_io_eprintln("Error: Invalid u32 input");
        syscall1(60, 1);
        __builtin_unreachable();
    }
    return result;
}

int64_t sm_std_io_read_i64(void) {
    const char* input = sm_std_io_readln();
    int64_t result;
    if (!sm_parse_i64(input, &result)) {
        sm_std_io_eprintln("Error: Invalid i64 input");
        syscall1(60, 1);
        __builtin_unreachable();
    }
    return result;
}

uint64_t sm_std_io_read_u64(void) {
    const char* input = sm_std_io_readln();
    uint64_t result;
    if (!sm_parse_u64(input, &result)) {
        sm_std_io_eprintln("Error: Invalid u64 input");
        syscall1(60, 1);
        __builtin_unreachable();
    }
    return result;
}