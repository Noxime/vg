(module binary
  "\00\61\73\6d\01\00\00\00\01\84\80\80\80\00\01\60"
  "\00\00\03\82\80\80\80\00\01\00\07\85\80\80\80\00"
  "\01\01\61\00\00\0a\88\80\80\80\00\01\82\80\80\80"
  "\00\00\0b"
)
(module binary
  "\00\61\73\6d\01\00\00\00\01\84\80\80\80\00\01\60"
  "\00\00\03\82\80\80\80\00\01\00\07\89\80\80\80\00"
  "\02\01\61\00\00\01\62\00\00\0a\88\80\80\80\00\01"
  "\82\80\80\80\00\00\0b"
)
(module binary
  "\00\61\73\6d\01\00\00\00\01\84\80\80\80\00\01\60"
  "\00\00\03\83\80\80\80\00\02\00\00\07\89\80\80\80"
  "\00\02\01\61\00\00\01\62\00\01\0a\8f\80\80\80\00"
  "\02\82\80\80\80\00\00\0b\82\80\80\80\00\00\0b"
)
(module binary
  "\00\61\73\6d\01\00\00\00\01\84\80\80\80\00\01\60"
  "\00\00\03\82\80\80\80\00\01\00\07\85\80\80\80\00"
  "\01\01\61\00\00\0a\88\80\80\80\00\01\82\80\80\80"
  "\00\00\0b"
)
(module binary
  "\00\61\73\6d\01\00\00\00\01\84\80\80\80\00\01\60"
  "\00\00\03\82\80\80\80\00\01\00\07\8d\80\80\80\00"
  "\03\01\61\00\00\01\62\00\00\01\63\00\00\0a\88\80"
  "\80\80\00\01\82\80\80\80\00\00\0b"
)
(module binary
  "\00\61\73\6d\01\00\00\00\01\85\80\80\80\00\01\60"
  "\01\7f\00\03\82\80\80\80\00\01\00\07\89\80\80\80"
  "\00\02\01\61\00\00\01\62\00\00\0a\88\80\80\80\00"
  "\01\82\80\80\80\00\00\0b"
)
(module binary
  "\00\61\73\6d\01\00\00\00\01\84\80\80\80\00\01\60"
  "\00\00\03\82\80\80\80\00\01\00\07\85\80\80\80\00"
  "\01\01\61\00\00\0a\88\80\80\80\00\01\82\80\80\80"
  "\00\00\0b"
)
(module binary
  "\00\61\73\6d\01\00\00\00\01\84\80\80\80\00\01\60"
  "\00\00\03\82\80\80\80\00\01\00\07\85\80\80\80\00"
  "\01\01\61\00\00\0a\88\80\80\80\00\01\82\80\80\80"
  "\00\00\0b"
)
(module binary
  "\00\61\73\6d\01\00\00\00\01\84\80\80\80\00\01\60"
  "\00\00\03\82\80\80\80\00\01\00\07\85\80\80\80\00"
  "\01\01\61\00\00\0a\88\80\80\80\00\01\82\80\80\80"
  "\00\00\0b"
)
(module binary
  "\00\61\73\6d\01\00\00\00\01\84\80\80\80\00\01\60"
  "\00\00\03\82\80\80\80\00\01\00\07\85\80\80\80\00"
  "\01\01\61\00\00\0a\88\80\80\80\00\01\82\80\80\80"
  "\00\00\0b"
)
(module binary
  "\00\61\73\6d\01\00\00\00\01\84\80\80\80\00\01\60"
  "\00\00\03\82\80\80\80\00\01\00\07\85\80\80\80\00"
  "\01\01\61\00\00\0a\88\80\80\80\00\01\82\80\80\80"
  "\00\00\0b"
)
(module $Func binary
  "\00\61\73\6d\01\00\00\00\01\86\80\80\80\00\01\60"
  "\01\7f\01\7f\03\82\80\80\80\00\01\00\07\85\80\80"
  "\80\00\01\01\65\00\00\0a\8e\80\80\80\00\01\88\80"
  "\80\80\00\00\20\00\41\01\6a\0f\0b"
)
(assert_return (invoke "e" (i32.const 42)) (i32.const 43))
(assert_return (invoke $Func "e" (i32.const 42)) (i32.const 43))
(module binary "\00\61\73\6d\01\00\00\00")
(module $Other1 binary "\00\61\73\6d\01\00\00\00")
(assert_return (invoke $Func "e" (i32.const 42)) (i32.const 43))
(assert_invalid
  (module binary
    "\00\61\73\6d\01\00\00\00\01\84\80\80\80\00\01\60"
    "\00\00\03\82\80\80\80\00\01\00\07\85\80\80\80\00"
    "\01\01\61\00\01\0a\88\80\80\80\00\01\82\80\80\80"
    "\00\00\0b"
  )
  "unknown function"
)
(assert_invalid
  (module binary
    "\00\61\73\6d\01\00\00\00\01\84\80\80\80\00\01\60"
    "\00\00\03\82\80\80\80\00\01\00\07\89\80\80\80\00"
    "\02\01\61\00\00\01\61\00\00\0a\88\80\80\80\00\01"
    "\82\80\80\80\00\00\0b"
  )
  "duplicate export name"
)
(assert_invalid
  (module binary
    "\00\61\73\6d\01\00\00\00\01\84\80\80\80\00\01\60"
    "\00\00\03\83\80\80\80\00\02\00\00\07\89\80\80\80"
    "\00\02\01\61\00\00\01\61\00\01\0a\8f\80\80\80\00"
    "\02\82\80\80\80\00\00\0b\82\80\80\80\00\00\0b"
  )
  "duplicate export name"
)
(assert_invalid
  (module binary
    "\00\61\73\6d\01\00\00\00\01\84\80\80\80\00\01\60"
    "\00\00\03\82\80\80\80\00\01\00\06\86\80\80\80\00"
    "\01\7f\00\41\00\0b\07\89\80\80\80\00\02\01\61\00"
    "\00\01\61\03\00\0a\88\80\80\80\00\01\82\80\80\80"
    "\00\00\0b"
  )
  "duplicate export name"
)
(assert_invalid
  (module binary
    "\00\61\73\6d\01\00\00\00\01\84\80\80\80\00\01\60"
    "\00\00\03\82\80\80\80\00\01\00\04\84\80\80\80\00"
    "\01\70\00\00\07\89\80\80\80\00\02\01\61\00\00\01"
    "\61\01\00\0a\88\80\80\80\00\01\82\80\80\80\00\00"
    "\0b"
  )
  "duplicate export name"
)
(assert_invalid
  (module binary
    "\00\61\73\6d\01\00\00\00\01\84\80\80\80\00\01\60"
    "\00\00\03\82\80\80\80\00\01\00\05\83\80\80\80\00"
    "\01\00\00\07\89\80\80\80\00\02\01\61\00\00\01\61"
    "\02\00\0a\88\80\80\80\00\01\82\80\80\80\00\00\0b"
  )
  "duplicate export name"
)
(module binary
  "\00\61\73\6d\01\00\00\00\06\86\80\80\80\00\01\7f"
  "\00\41\00\0b\07\85\80\80\80\00\01\01\61\03\00"
)
(module binary
  "\00\61\73\6d\01\00\00\00\06\86\80\80\80\00\01\7f"
  "\00\41\00\0b\07\89\80\80\80\00\02\01\61\03\00\01"
  "\62\03\00"
)
(module binary
  "\00\61\73\6d\01\00\00\00\06\8b\80\80\80\00\02\7f"
  "\00\41\00\0b\7f\00\41\00\0b\07\89\80\80\80\00\02"
  "\01\61\03\00\01\62\03\01"
)
(module binary
  "\00\61\73\6d\01\00\00\00\06\86\80\80\80\00\01\7f"
  "\00\41\00\0b\07\85\80\80\80\00\01\01\61\03\00"
)
(module binary
  "\00\61\73\6d\01\00\00\00\06\86\80\80\80\00\01\7f"
  "\00\41\00\0b\07\85\80\80\80\00\01\01\61\03\00"
)
(module binary
  "\00\61\73\6d\01\00\00\00\06\86\80\80\80\00\01\7f"
  "\00\41\00\0b\07\85\80\80\80\00\01\01\61\03\00"
)
(module binary
  "\00\61\73\6d\01\00\00\00\06\86\80\80\80\00\01\7f"
  "\00\41\00\0b\07\85\80\80\80\00\01\01\61\03\00"
)
(module binary
  "\00\61\73\6d\01\00\00\00\06\86\80\80\80\00\01\7f"
  "\00\41\00\0b\07\85\80\80\80\00\01\01\61\03\00"
)
(module binary
  "\00\61\73\6d\01\00\00\00\06\86\80\80\80\00\01\7f"
  "\00\41\00\0b\07\85\80\80\80\00\01\01\61\03\00"
)
(module $Global binary
  "\00\61\73\6d\01\00\00\00\06\86\80\80\80\00\01\7f"
  "\00\41\2a\0b\07\85\80\80\80\00\01\01\65\03\00"
)
(assert_return (get "e") (i32.const 42))
(assert_return (get $Global "e") (i32.const 42))
(module binary "\00\61\73\6d\01\00\00\00")
(module $Other2 binary "\00\61\73\6d\01\00\00\00")
(assert_return (get $Global "e") (i32.const 42))
(assert_invalid
  (module binary
    "\00\61\73\6d\01\00\00\00\06\86\80\80\80\00\01\7f"
    "\00\41\00\0b\07\85\80\80\80\00\01\01\61\03\01"
  )
  "unknown global"
)
(assert_invalid
  (module binary
    "\00\61\73\6d\01\00\00\00\06\86\80\80\80\00\01\7f"
    "\00\41\00\0b\07\89\80\80\80\00\02\01\61\03\00\01"
    "\61\03\00"
  )
  "duplicate export name"
)
(assert_invalid
  (module binary
    "\00\61\73\6d\01\00\00\00\06\8b\80\80\80\00\02\7f"
    "\00\41\00\0b\7f\00\41\00\0b\07\89\80\80\80\00\02"
    "\01\61\03\00\01\61\03\01"
  )
  "duplicate export name"
)
(assert_invalid
  (module binary
    "\00\61\73\6d\01\00\00\00\01\84\80\80\80\00\01\60"
    "\00\00\03\82\80\80\80\00\01\00\06\86\80\80\80\00"
    "\01\7f\00\41\00\0b\07\89\80\80\80\00\02\01\61\03"
    "\00\01\61\00\00\0a\88\80\80\80\00\01\82\80\80\80"
    "\00\00\0b"
  )
  "duplicate export name"
)
(assert_invalid
  (module binary
    "\00\61\73\6d\01\00\00\00\04\84\80\80\80\00\01\70"
    "\00\00\06\86\80\80\80\00\01\7f\00\41\00\0b\07\89"
    "\80\80\80\00\02\01\61\03\00\01\61\01\00"
  )
  "duplicate export name"
)
(assert_invalid
  (module binary
    "\00\61\73\6d\01\00\00\00\05\83\80\80\80\00\01\00"
    "\00\06\86\80\80\80\00\01\7f\00\41\00\0b\07\89\80"
    "\80\80\00\02\01\61\03\00\01\61\02\00"
  )
  "duplicate export name"
)
(module binary
  "\00\61\73\6d\01\00\00\00\04\84\80\80\80\00\01\70"
  "\00\00\07\85\80\80\80\00\01\01\61\01\00"
)
(module binary
  "\00\61\73\6d\01\00\00\00\04\84\80\80\80\00\01\70"
  "\00\00\07\89\80\80\80\00\02\01\61\01\00\01\62\01"
  "\00"
)
(module binary
  "\00\61\73\6d\01\00\00\00\04\84\80\80\80\00\01\70"
  "\00\00\07\85\80\80\80\00\01\01\61\01\00"
)
(module binary
  "\00\61\73\6d\01\00\00\00\04\85\80\80\80\00\01\70"
  "\01\00\01\07\85\80\80\80\00\01\01\61\01\00"
)
(module binary
  "\00\61\73\6d\01\00\00\00\04\84\80\80\80\00\01\70"
  "\00\00\07\85\80\80\80\00\01\01\61\01\00"
)
(module binary
  "\00\61\73\6d\01\00\00\00\04\85\80\80\80\00\01\70"
  "\01\00\01\07\85\80\80\80\00\01\01\61\01\00"
)
(module binary
  "\00\61\73\6d\01\00\00\00\04\84\80\80\80\00\01\70"
  "\00\00\07\85\80\80\80\00\01\01\61\01\00"
)
(module binary
  "\00\61\73\6d\01\00\00\00\04\85\80\80\80\00\01\70"
  "\01\00\01\07\85\80\80\80\00\01\01\61\01\00"
)
(module binary
  "\00\61\73\6d\01\00\00\00\04\84\80\80\80\00\01\70"
  "\00\00\07\85\80\80\80\00\01\01\61\01\00"
)
(module binary
  "\00\61\73\6d\01\00\00\00\04\85\80\80\80\00\01\70"
  "\01\00\01\07\85\80\80\80\00\01\01\61\01\00"
)
(module binary
  "\00\61\73\6d\01\00\00\00\04\84\80\80\80\00\01\70"
  "\00\00\07\85\80\80\80\00\01\01\61\01\00"
)
(module binary
  "\00\61\73\6d\01\00\00\00\04\85\80\80\80\00\01\70"
  "\01\00\01\07\85\80\80\80\00\01\01\61\01\00"
)
(module binary
  "\00\61\73\6d\01\00\00\00\04\84\80\80\80\00\01\70"
  "\00\00\07\85\80\80\80\00\01\01\61\01\00"
)
(module binary
  "\00\61\73\6d\01\00\00\00\04\85\80\80\80\00\01\70"
  "\01\00\01\07\85\80\80\80\00\01\01\61\01\00"
)
(assert_invalid
  (module binary
    "\00\61\73\6d\01\00\00\00\04\84\80\80\80\00\01\70"
    "\00\00\07\85\80\80\80\00\01\01\61\01\01"
  )
  "unknown table"
)
(assert_invalid
  (module binary
    "\00\61\73\6d\01\00\00\00\04\84\80\80\80\00\01\70"
    "\00\00\07\89\80\80\80\00\02\01\61\01\00\01\61\01"
    "\00"
  )
  "duplicate export name"
)
(assert_invalid
  (module binary
    "\00\61\73\6d\01\00\00\00\01\84\80\80\80\00\01\60"
    "\00\00\03\82\80\80\80\00\01\00\04\84\80\80\80\00"
    "\01\70\00\00\07\89\80\80\80\00\02\01\61\01\00\01"
    "\61\00\00\0a\88\80\80\80\00\01\82\80\80\80\00\00"
    "\0b"
  )
  "duplicate export name"
)
(assert_invalid
  (module binary
    "\00\61\73\6d\01\00\00\00\04\84\80\80\80\00\01\70"
    "\00\00\06\86\80\80\80\00\01\7f\00\41\00\0b\07\89"
    "\80\80\80\00\02\01\61\01\00\01\61\03\00"
  )
  "duplicate export name"
)
(assert_invalid
  (module binary
    "\00\61\73\6d\01\00\00\00\04\84\80\80\80\00\01\70"
    "\00\00\05\83\80\80\80\00\01\00\00\07\89\80\80\80"
    "\00\02\01\61\01\00\01\61\02\00"
  )
  "duplicate export name"
)
(module binary
  "\00\61\73\6d\01\00\00\00\05\83\80\80\80\00\01\00"
  "\00\07\85\80\80\80\00\01\01\61\02\00"
)
(module binary
  "\00\61\73\6d\01\00\00\00\05\83\80\80\80\00\01\00"
  "\00\07\89\80\80\80\00\02\01\61\02\00\01\62\02\00"
)
(module binary
  "\00\61\73\6d\01\00\00\00\05\83\80\80\80\00\01\00"
  "\00\07\85\80\80\80\00\01\01\61\02\00"
)
(module binary
  "\00\61\73\6d\01\00\00\00\05\84\80\80\80\00\01\01"
  "\00\01\07\85\80\80\80\00\01\01\61\02\00"
)
(module binary
  "\00\61\73\6d\01\00\00\00\05\83\80\80\80\00\01\00"
  "\00\07\85\80\80\80\00\01\01\61\02\00"
)
(module binary
  "\00\61\73\6d\01\00\00\00\05\84\80\80\80\00\01\01"
  "\00\01\07\85\80\80\80\00\01\01\61\02\00"
)
(module binary
  "\00\61\73\6d\01\00\00\00\05\83\80\80\80\00\01\00"
  "\00\07\85\80\80\80\00\01\01\61\02\00"
)
(module binary
  "\00\61\73\6d\01\00\00\00\05\84\80\80\80\00\01\01"
  "\00\01\07\85\80\80\80\00\01\01\61\02\00"
)
(module binary
  "\00\61\73\6d\01\00\00\00\05\83\80\80\80\00\01\00"
  "\00\07\85\80\80\80\00\01\01\61\02\00"
)
(module binary
  "\00\61\73\6d\01\00\00\00\05\84\80\80\80\00\01\01"
  "\00\01\07\85\80\80\80\00\01\01\61\02\00"
)
(module binary
  "\00\61\73\6d\01\00\00\00\05\83\80\80\80\00\01\00"
  "\00\07\85\80\80\80\00\01\01\61\02\00"
)
(module binary
  "\00\61\73\6d\01\00\00\00\05\84\80\80\80\00\01\01"
  "\00\01\07\85\80\80\80\00\01\01\61\02\00"
)
(module binary
  "\00\61\73\6d\01\00\00\00\05\83\80\80\80\00\01\00"
  "\00\07\85\80\80\80\00\01\01\61\02\00"
)
(module binary
  "\00\61\73\6d\01\00\00\00\05\84\80\80\80\00\01\01"
  "\00\01\07\85\80\80\80\00\01\01\61\02\00"
)
(assert_invalid
  (module binary
    "\00\61\73\6d\01\00\00\00\05\83\80\80\80\00\01\00"
    "\00\07\85\80\80\80\00\01\01\61\02\01"
  )
  "unknown memory"
)
(assert_invalid
  (module binary
    "\00\61\73\6d\01\00\00\00\05\83\80\80\80\00\01\00"
    "\00\07\89\80\80\80\00\02\01\61\02\00\01\61\02\00"
  )
  "duplicate export name"
)
(assert_invalid
  (module binary
    "\00\61\73\6d\01\00\00\00\01\84\80\80\80\00\01\60"
    "\00\00\03\82\80\80\80\00\01\00\05\83\80\80\80\00"
    "\01\00\00\07\89\80\80\80\00\02\01\61\02\00\01\61"
    "\00\00\0a\88\80\80\80\00\01\82\80\80\80\00\00\0b"
  )
  "duplicate export name"
)
(assert_invalid
  (module binary
    "\00\61\73\6d\01\00\00\00\05\83\80\80\80\00\01\00"
    "\00\06\86\80\80\80\00\01\7f\00\41\00\0b\07\89\80"
    "\80\80\00\02\01\61\02\00\01\61\03\00"
  )
  "duplicate export name"
)
(assert_invalid
  (module binary
    "\00\61\73\6d\01\00\00\00\04\84\80\80\80\00\01\70"
    "\00\00\05\83\80\80\80\00\01\00\00\07\89\80\80\80"
    "\00\02\01\61\02\00\01\61\01\00"
  )
  "duplicate export name"
)
