; ModuleID = 'test'
source_filename = "test"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"

define void @main() {
entry:
}

define i64 @jia(i64 %0) {
entry:
  %"0" = add i64 %0, 1
  %jie2guo3 = alloca i64, align 8
  store i64 %"0", ptr %jie2guo3, align 8
  %1 = load i64, ptr %jie2guo3, align 8
  ret i64 %1
}

define i64 @jian(i64 %0) {
entry:
  %"0" = sub i64 %0, 1
  ret i64 %"0"
}
