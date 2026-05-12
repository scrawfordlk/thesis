define i64 @main() {
entry:
  %t1 = add i64 40, 2    ; 42
  %t2 = mul i64 3, %t1   ; 126
  %t3 = udiv i64 %t2, 2  ; 63 
  %t4 = urem i64 %t3, 40 ; 23
  %t5 = sub i64 %t4, 2   ; 21
  %t6 = mul i64 2, %t5   ; 42
  ret i64 %t6
}
