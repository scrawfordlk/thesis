define i64 @main() {
entry:
  %cmp = icmp ult i64 4, 2
  br i1 %cmp, label %then, label %else

then:
  ret i64 43

else:
  ret i64 42
}
