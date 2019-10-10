(declare-datatypes () (
  (Type
  int
  bool
  (func (in LType) (out LType)))

  (LType nil (cons (head Type) (tail LType)))
  (Option (none) (some (thing LType)))
))

(define-fun cons_int ((x LType)) LType (cons int x))
(define-fun cons_bool ((x LType)) LType (cons bool x))
(define-const int1 LType (cons_int nil))
(define-const int2 LType (cons_int int1))
(define-const int3 LType (cons_int int2))
(define-const bool1 LType (cons_int nil))

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


(push)
(assert (not (distinct int bool)))
(assert (not (= (append nil nil) nil)))
(assert (not (= (append nil int1) int1)))
(assert (not (= (append int1 nil) int1)))
(assert (not (= (append int2 int1) int3)))
(assert (not (= (append int1 int2) int3)))
(assert (not (= (rev int2) int2)))
(assert (not (= (cons_bool (cons_int nil)) (rev (cons_int (cons_bool nil))))))
(assert (not (= (cons_int (cons_bool nil)) (rev (cons_bool (cons_int nil))))))
(assert (not (= (last (cons_bool (cons_bool (cons_int nil)))) int)))
(assert (not (is_some (peel int1 int1))))
(assert (not (not (is_some (peel int2 int1)))))
(assert (not (not (is_some (peel int3 int1)))))
(assert (not (is_some (peel int1 int2))))
(assert (not (is_some (peel int2 int2))))
(assert (not (not (is_some (peel int3 int2)))))
(assert (not (is_some (peel int1 int3))))
(assert (not (is_some (peel int2 int3))))
(assert (not (is_some (peel int3 int3))))
(check-sat)
; This should be *unsat* -- the negation of all of the above assertions should
; hold deterministically; there is no variable to be instantiated by Z3
(pop)



; I'm not sure why this exists, but it does. Here be dragons.
(define-const absurd LType (thing none))

; Convert non-functions to nullary functions
(define-fun to_func ((f Type)) Type
  (match f (
    ((func I O) f)
    (int (func nil int1))
    (bool (func nil bool1))
  ))
)

(define-fun-rec apply ((f Type) (stack LType)) Option
  (match f (
    ((func I O) (match (peel I stack) (
      ((some S) (some (append O S)))
      (none none)
    )))
    (X (apply (to_func f) stack))
  ))
)

(define-fun-rec apply_many ((fs LType) (stack Option)) Option
  (match stack (
    ((some S) (match fs (
      ((cons f T) (apply_many T (apply f S)))
      (nil stack)
    )))
    (none none)
  ))
)

(define-const add Type (func int2 int1))
(assert (= (apply add int2) (some int1)))
(assert (= none (apply add nil)))

(define-const add1 LType (cons add nil))
(define-const add3 LType (cons add (cons add add1)))

(declare-const test_stack LType)
(declare-const test_stack_out Option)

; (assert (= test_stack (append int2 int2)))

(assert (= test_stack_out (apply_many add1 (some test_stack))))
(assert (not (= none test_stack_out)))

(check-sat)
(get-model)