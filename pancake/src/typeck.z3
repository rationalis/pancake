;(declare-datatypes Type)
(declare-datatypes () (
  (Type
  int
  bool
  (func (in LType) (out LType)))

  (LType nil (cons (head Type) (tail LType)))
))

(declare-datatypes () ((Option (none) (some (thing LType)))))

(define-fun cons_int ((x LType)) LType (cons int x))
(define-fun cons_bool ((x LType)) LType (cons bool x))
(define-const int1 LType (cons_int nil))
(define-const int2 LType (cons_int int1))
(define-const int3 LType (cons_int int2))

(define-const i Int 1)

(define-fun-rec peel ((prefix LType) (seq LType)) Option
  (match prefix (
    ((cons H T)
      (match seq (
        ((cons H2 T2)
          (ite (= H H2) (peel T T2) none))
        (nil none)
      )))
    (nil (some seq))
)))

(define-fun-rec append ((L1 LType) (L2 LType)) LType
  (match L1 (
    ((cons H T)
     (cons H (append T L2)))
    (nil L2)
  ))
)

(define-fun-rec rev ((L LType)) LType
  (match L (
    ((cons H T)
      (append (rev T) (cons H nil)))
    (nil nil)
  ))
)

(define-fun last ((L LType)) Type
  (head (rev L))
)

(define-fun is_some ((O Option)) Bool
  (match O (
    ((some V) true)
    (none false)
  ))
)

; TODO
; (define-fun compose_fns ((T1 Type) (T2 Type)) Type
;   (match T1 (
;     (func I O)
;     (int)
;     (bool)
;   )
; )



; General tests
(assert (distinct int bool))

; Test for append
(assert (= (append nil nil) nil))

(assert (= (append nil int1) int1))
(assert (= (append int1 nil) int1))

(assert (= (append int2 int1) int3))
(assert (= (append int1 int2) int3))

; Test for rev
(assert (= (rev int2) int2))
(assert (= (cons_bool (cons_int nil)) (rev (cons_int (cons_bool nil)))))
(assert (= (cons_int (cons_bool nil)) (rev (cons_bool (cons_int nil)))))

; Test for last
(assert (= (last (cons_bool (cons_bool (cons_int nil)))) int))

; Test for peel
(assert (is_some (peel int1 int1)))
(assert (not (is_some (peel int2 int1))))
(assert (not (is_some (peel int3 int1))))

(assert (is_some (peel int1 int2)))
(assert (is_some (peel int2 int2)))
(assert (not (is_some (peel int3 int2))))

(assert (is_some (peel int1 int3)))
(assert (is_some (peel int2 int3)))
(assert (is_some (peel int3 int3)))

; I'm not sure why this exists, but it does. Here be dragons.
(define-const absurd LType (thing none))



(check-sat)
(get-model)