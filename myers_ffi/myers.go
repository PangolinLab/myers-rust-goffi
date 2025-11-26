package myers_ffi

/*
	#cgo CFLAGS: -I${SRCDIR}/include
	#cgo LDFLAGS: -lkernel32 -lntdll -luserenv -lws2_32 -ldbghelp -L${SRCDIR}/bin -lmyers
	#include <stdlib.h>
	#include <myers_interface.h>
*/
import "C"

import (
	"unsafe"
)

type EditOp int

const (
	Equal EditOp = iota
	Delete
	Insert
)

type EditRecord struct {
	Op   EditOp
	Line string
}

func GetDiffs(oldLines, newLines []string) []EditRecord {
	cOld := make([]*C.char, len(oldLines)+1)
	cNew := make([]*C.char, len(newLines)+1)
	for i, s := range oldLines {
		cOld[i] = C.CString(s)
	}
	cOld[len(oldLines)] = nil
	for i, s := range newLines {
		cNew[i] = C.CString(s)
	}
	cNew[len(newLines)] = nil
	defer func() {
		for _, p := range cOld {
			if p != nil {
				C.free(unsafe.Pointer(p))
			}
		}
		for _, p := range cNew {
			if p != nil {
				C.free(unsafe.Pointer(p))
			}
		}
	}()

	rec := C.diff_lines((**C.char)(&cOld[0]), (**C.char)(&cNew[0]))
	if rec == nil {
		return nil
	}
	defer C.free_diff(rec)

	var goRec []EditRecord
	for p := rec; ; p = (*C.EditRecord)(unsafe.Pointer(uintptr(unsafe.Pointer(p)) + unsafe.Sizeof(*p))) {
		if p.line == nil {
			break
		}
		goRec = append(goRec, EditRecord{
			Op:   EditOp(p.op),
			Line: C.GoString(p.line),
		})
	}
	return goRec
}
