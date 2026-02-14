#ifndef SM_IO_H
#define SM_IO_H

#include "freestanding.h"

void sm_std_io_print(const char* s);
void sm_std_io_println(const char* s);
void sm_std_io_eprint(const char* s);
void sm_std_io_eprintln(const char* s);

void sm_std_io_print_i64(int64_t n);
void sm_std_io_println_i64(int64_t n);
void sm_std_io_print_u64(uint64_t n);
void sm_std_io_println_u64(uint64_t n);
void sm_std_io_print_i128(int128_t n);
void sm_std_io_println_i128(int128_t n);
void sm_std_io_print_u128(uint128_t n);
void sm_std_io_println_u128(uint128_t n);

void sm_std_io_print_bool(bool b);
void sm_std_io_println_bool(bool b);

const char* sm_std_io_readln(void);

int8_t sm_std_io_read_i8(void);
uint8_t sm_std_io_read_u8(void);
int16_t sm_std_io_read_i16(void);
uint16_t sm_std_io_read_u16(void);
int32_t sm_std_io_read_i32(void);
uint32_t sm_std_io_read_u32(void);
int64_t sm_std_io_read_i64(void);
uint64_t sm_std_io_read_u64(void);

#endif