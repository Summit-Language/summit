#include "io.h"
#include "string_utils.h"

int8_t sm_std_io_read_i8(void) {
    const char* input = sm_std_io_readln();
    int8_t result;
    if (!sm_parse_i8(input, &result)) {
        sm_std_io_eprintln("Error: Invalid i8 input");
        syscall1(SYS_exit, 1);
        __builtin_unreachable();
    }
    return result;
}

uint8_t sm_std_io_read_u8(void) {
    const char* input = sm_std_io_readln();
    uint8_t result;
    if (!sm_parse_u8(input, &result)) {
        sm_std_io_eprintln("Error: Invalid u8 input");
        syscall1(SYS_exit, 1);
        __builtin_unreachable();
    }
    return result;
}

int16_t sm_std_io_read_i16(void) {
    const char* input = sm_std_io_readln();
    int16_t result;
    if (!sm_parse_i16(input, &result)) {
        sm_std_io_eprintln("Error: Invalid i16 input");
        syscall1(SYS_exit, 1);
        __builtin_unreachable();
    }
    return result;
}

uint16_t sm_std_io_read_u16(void) {
    const char* input = sm_std_io_readln();
    uint16_t result;
    if (!sm_parse_u16(input, &result)) {
        sm_std_io_eprintln("Error: Invalid u16 input");
        syscall1(SYS_exit, 1);
        __builtin_unreachable();
    }
    return result;
}

int32_t sm_std_io_read_i32(void) {
    const char* input = sm_std_io_readln();
    int32_t result;
    if (!sm_parse_i32(input, &result)) {
        sm_std_io_eprintln("Error: Invalid i32 input");
        syscall1(SYS_exit, 1);
        __builtin_unreachable();
    }
    return result;
}

uint32_t sm_std_io_read_u32(void) {
    const char* input = sm_std_io_readln();
    uint32_t result;
    if (!sm_parse_u32(input, &result)) {
        sm_std_io_eprintln("Error: Invalid u32 input");
        syscall1(SYS_exit, 1);
        __builtin_unreachable();
    }
    return result;
}

int64_t sm_std_io_read_i64(void) {
    const char* input = sm_std_io_readln();
    int64_t result;
    if (!sm_parse_i64(input, &result)) {
        sm_std_io_eprintln("Error: Invalid i64 input");
        syscall1(SYS_exit, 1);
        __builtin_unreachable();
    }
    return result;
}

uint64_t sm_std_io_read_u64(void) {
    const char* input = sm_std_io_readln();
    uint64_t result;
    if (!sm_parse_u64(input, &result)) {
        sm_std_io_eprintln("Error: Invalid u64 input");
        syscall1(SYS_exit, 1);
        __builtin_unreachable();
    }
    return result;
}