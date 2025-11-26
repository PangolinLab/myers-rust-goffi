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

EditRecord * diff_lines(const char **old_lines, const char **new_lines);
void free_diff(EditRecord *records);

#ifdef __cplusplus
}
#endif

#endif // MYERS_INTERFACE_H
