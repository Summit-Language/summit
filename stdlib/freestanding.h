#ifndef FREESTANDING_H
#define FREESTANDING_H

typedef unsigned char uint8_t;
typedef signed char int8_t;
typedef unsigned short uint16_t;
typedef signed short int16_t;
typedef unsigned int uint32_t;
typedef signed int int32_t;
typedef unsigned long long uint64_t;
typedef signed long long int64_t;
typedef unsigned __int128 uint128_t;
typedef __int128 int128_t;

typedef int bool;

typedef unsigned long size_t;
typedef long ssize_t;

#define TRUE 1
#define FALSE 0

#define NULL ((void*)0)

#define SYS_read 0
#define SYS_write 1
#define SYS_exit 60

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

#define STDIN_FILENO 0
#define STDOUT_FILENO 1
#define STDERR_FILENO 2

#endif