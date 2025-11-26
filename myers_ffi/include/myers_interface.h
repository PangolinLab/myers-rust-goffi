#ifndef MYERS_INTERFACE_H
#define MYERS_INTERFACE_H

#ifdef __cplusplus
extern "C" {
#endif

typedef enum { Equal = 0, Delete = 1, Insert = 2 } EditOp;

typedef struct {
    EditOp op;
    const char *line;
} EditRecord;

/// 计算两个文本的差异，返回 null 结尾的 EditRecord 数组
EditRecord * diff_lines(const char **old_lines, const char **new_lines);

/// 释放 diff_lines 返回的 EditRecord 数组
void free_diff(EditRecord *records);

/// 根据 diff 应用到旧文本，生成新文本，返回 null 结尾的 C 字符串数组
char ** apply_diff(const char **old_lines, const EditRecord *records);

/// 释放 apply_diff 返回的 C 字符串数组
void free_applied(char **lines);

#ifdef __cplusplus
}
#endif

#endif // MYERS_INTERFACE_H
