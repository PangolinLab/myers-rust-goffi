use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[repr(C)]
pub enum EditOp {
    Equal = 0,
    Delete = 1,
    Insert = 2,
}

#[repr(C)]
pub struct EditRecord {
    pub op: EditOp,
    pub line: *const c_char,
}

#[no_mangle]
pub extern "C" fn diff_lines(
    old_lines: *const *const c_char,
    new_lines: *const *const c_char,
) -> *mut EditRecord {
    let mut old: Vec<String> = Vec::new();
    let mut p = old_lines;
    unsafe {
        while !(*p).is_null() {
            old.push(CStr::from_ptr(*p).to_string_lossy().into_owned());
            p = p.add(1);
        }
    }

    let mut new: Vec<String> = Vec::new();
    let mut q = new_lines;
    unsafe {
        while !(*q).is_null() {
            new.push(CStr::from_ptr(*q).to_string_lossy().into_owned());
            q = q.add(1);
        }
    }

    let script = myers_diff(&old, &new);

    let mut recs: Vec<EditRecord> = Vec::with_capacity(script.len() + 1);
    for (op, line) in script {
        let c_line = CString::new(line).unwrap();
        recs.push(EditRecord {
            op: match op.as_str() {
                "Equal" => EditOp::Equal,
                "Delete" => EditOp::Delete,
                "Insert" => EditOp::Insert,
                _ => EditOp::Equal,
            },
            line: c_line.into_raw(),
        });
    }
    recs.push(EditRecord { op: EditOp::Equal, line: std::ptr::null() });
    let ptr = recs.as_mut_ptr();
    std::mem::forget(recs);
    ptr
}

#[no_mangle]
pub extern "C" fn free_diff(records: *mut EditRecord) {
    if records.is_null() {
        return;
    }
    unsafe {
        let mut p = records;
        while !(*p).line.is_null() {
            let _ = CString::from_raw((*p).line as *mut c_char);
            p = p.add(1);
        }
        let _ = Vec::from_raw_parts(records, 0, 0);
    }
}

#[no_mangle]
pub extern "C" fn apply_diff(
    old_lines: *const *const c_char,
    records: *const EditRecord,
) -> *mut *mut c_char {
    // 1. 读取 old_lines
    let mut old: Vec<String> = Vec::new();
    unsafe {
        let mut p = old_lines;
        while !(*p).is_null() {
            old.push(CStr::from_ptr(*p).to_string_lossy().into_owned());
            p = p.add(1);
        }
    }

    // 2. 遍历 EditRecord 构建新文本
    let mut new_lines: Vec<CString> = Vec::new();
    let mut old_idx = 0;

    unsafe {
        let mut r = records;
        while !(*r).line.is_null() {
            match (*r).op {
                EditOp::Equal => {
                    if old_idx < old.len() {
                        new_lines.push(CString::new(old[old_idx].clone()).unwrap());
                        old_idx += 1;
                    }
                }
                EditOp::Insert => {
                    new_lines.push(CString::new(CStr::from_ptr((*r).line).to_string_lossy().into_owned()).unwrap());
                }
                EditOp::Delete => {
                    old_idx += 1; // 跳过旧行
                }
            }
            r = r.add(1);
        }
    }

    // 3. 转为 C 字符串数组
    let mut c_arr: Vec<*mut c_char> = new_lines.iter_mut().map(|s| s.as_ptr() as *mut c_char).collect();
    c_arr.push(std::ptr::null_mut());

    let ptr = c_arr.as_mut_ptr();
    std::mem::forget(new_lines);
    std::mem::forget(c_arr);
    ptr
}

/// free apply_diff 返回的数组
#[no_mangle]
pub extern "C" fn free_applied(lines: *mut *mut c_char) {
    if lines.is_null() {
        return;
    }
    unsafe {
        let mut p = lines;
        while !(*p).is_null() {
            let _ = CString::from_raw(*p);
            p = p.add(1);
        }
        let _ = Vec::from_raw_parts(lines, 0, 0);
    }
}

fn myers_diff(old: &[String], new: &[String]) -> Vec<(String, String)> {
    let n = old.len();
    let m = new.len();
    let max = n + m;
    let mut v = std::collections::HashMap::<isize, usize>::new();
    v.insert(1, 0);
    let mut trace: Vec<std::collections::HashMap<isize, usize>> = Vec::new();

    for d in 0..=max {
        let mut v_next = v.clone();
        for k in (-((d as isize))..=(d as isize)).step_by(2) {
            let x = if k == - (d as isize)
                || (k != (d as isize) && v.get(&(k - 1)).cloned().unwrap_or(0) < v.get(&(k + 1)).cloned().unwrap_or(0))
            {
                v.get(&(k + 1)).cloned().unwrap_or(0)
            } else {
                v.get(&(k - 1)).cloned().unwrap_or(0) + 1
            };
            let mut y = (x as isize - k) as usize;
            let mut x_mut = x;
            while x_mut < n && y < m && old[x_mut] == new[y] {
                x_mut += 1;
                y += 1;
            }
            v_next.insert(k, x_mut);
            if x_mut >= n && y >= m {
                trace.push(v_next.clone());
                return backtrack(old, new, &trace, d);
            }
        }
        trace.push(v_next.clone());
        v = v_next;
    }
    panic!("diff failed");
}

fn backtrack(
    old: &[String],
    new: &[String],
    trace: &Vec<std::collections::HashMap<isize, usize>>,
    mut d: usize,
) -> Vec<(String, String)> {
    let mut result = Vec::new();
    let mut x = old.len();
    let mut y = new.len();

    while d > 0 {
        let v = &trace[d];
        let k = x as isize - y as isize;
        let prev_k = if k == - (d as isize)
            || (k != (d as isize) && v.get(&(k - 1)).cloned().unwrap_or(0) < v.get(&(k + 1)).cloned().unwrap_or(0))
        {
            k + 1
        } else {
            k - 1
        };
        let prev_vx = v.get(&prev_k).cloned().unwrap_or(0);
        let prev_x = prev_vx;
        let prev_y = (prev_x as isize - prev_k) as usize;

        let mut cur_x = prev_x;
        let mut cur_y = prev_y;

        while cur_x < x && cur_y < y {
            result.push(("Equal".to_string(), old[cur_x].clone()));
            cur_x += 1;
            cur_y += 1;
        }

        if cur_x < x {
            result.push(("Delete".to_string(), old[cur_x].clone()));
            cur_x += 1;
        } else if cur_y < y {
            result.push(("Insert".to_string(), new[cur_y].clone()));
            cur_y += 1;
        }

        x = prev_x;
        y = prev_y;
        d -= 1;
    }

    while x > 0 && y > 0 {
        if old[x - 1] == new[y - 1] {
            result.push(("Equal".to_string(), old[x - 1].clone()));
            x -= 1;
            y -= 1;
        } else {
            break;
        }
    }
    result.reverse();
    result
}
