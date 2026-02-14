#include "string_utils.h"

size_t sm_strlen(const char* s) {
    size_t len = 0;
    while (s[len]) len++;
    return len;
}

void sm_int_to_string(int64_t num, char* buf) {
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

void sm_uint64_to_string(uint64_t num, char* buf) {
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

void sm_int128_to_string(int128_t num, char* buf) {
    int i = 0;
    int is_negative = 0;
    uint128_t unsigned_num;

    if (num == 0) {
        buf[i++] = '0';
        buf[i] = '\0';
        return;
    }

    if (num < 0) {
        is_negative = 1;
        unsigned_num = (uint128_t)(-(num + 1)) + 1;
    } else {
        unsigned_num = (uint128_t)num;
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

void sm_uint128_to_string(uint128_t num, char* buf) {
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

int sm_parse_i8(const char* str, int8_t* out) {
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

int sm_parse_u8(const char* str, uint8_t* out) {
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

int sm_parse_i16(const char* str, int16_t* out) {
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

int sm_parse_u16(const char* str, uint16_t* out) {
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

int sm_parse_i32(const char* str, int32_t* out) {
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

int sm_parse_u32(const char* str, uint32_t* out) {
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

int sm_parse_i64(const char* str, int64_t* out) {
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

int sm_parse_u64(const char* str, uint64_t* out) {
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