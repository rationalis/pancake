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

(define-fun is_prefix ((prefix LType) (seq LType)) Bool
  (exists ((suffix LType))
    (= seq (append prefix suffix)))
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



(define-fun peel_prop_holds ((prefix LType) (seq LType)) Bool
  (let ((suffix (peel prefix seq)))
    (xor
      (= none suffix)
      (= seq (append prefix (thing suffix))))))

(push)

; I'm not sure why this is *necessary*, but it formalizes the pseudo-inverse
; relationship between append and peel. Not doing so leads to incorrect answers,
; somehow.
(assert (exists ((prefix LType) (seq LType))
  (not (peel_prop_holds prefix seq))
))

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

; I'm not sure why this exists, but it does. Here be dragons.
(define-const absurd LType (thing none))

(pop)




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

(push)
(define-const add Type (func int2 int1))
(assert (= (apply add int2) (some int1)))
(assert (= none (apply add nil)))

(define-const add1 LType (cons add nil))
(define-const add3 LType (cons add (cons add add1)))

(declare-const test_stack LType)
(declare-const test_stack_out Option)

(assert (= test_stack_out (apply_many add3 (some test_stack))))
(assert (not (= none test_stack_out)))

(check-sat)
(get-model)
(pop)

(define-const add Type (func int2 int1))
(assert (= (apply add int2) (some int1)))
(assert (= none (apply add nil)))

(define-const add1 LType (cons add nil))
(define-const add3 LType (cons add (cons add add1)))


(define-fun helper ((I2 LType) (O1 LType)) Option
  (peel (rev O1) (rev I2))
)

; O1: B B I2: A A B B
; O1: A A B B I2: B B

(define-fun-rec compose_pair_IO
  ((I1 LType) (I2 LType) (O1 LType) (O2 LType)) Option

  (let ((P1 (helper I2 O1)) (P2 (helper O1 I2)))
    (match P1 (
      ((some p1) (some (cons (func (append (rev p1) I1) O2) nil)))
      (none (match P2 (
        ((some p2) (some (cons (func I1 (append (rev p2) O2)) nil)))
        (none none)
      )))
    ))
  )
)

(define-fun-rec compose_pair ((f1 Type) (f2 Type)) Option
  (match f1 (
    ((func I1 O1) (match f2 (
      ((func I2 O2) (compose_pair_IO I1 I2 O1 O2))
      (Y (compose_pair f1 (to_func f2)))
    )))
    (X (compose_pair (to_func f1) f2))
  ))
)

(define-fun helper2 ((H Type) (tail Option)) Option
  (match tail (
    ((some ty) (compose_pair H (head ty)))
    (none none)
  ))
)

(define-fun-rec compose ((fs LType)) Option
  (match fs (
    ((cons H T) (helper2 H (compose T)))
    (nil (some (cons (func nil nil) nil)))
  ))
)

; (assert (forall ((fs LType))
;   (exists ((input_stack LType) (output_stack Option))
;       (and
;         (= output_stack (apply_many fs (some input_stack)))
;         (not (= none output_stack))
;         (=
;           (apply (func input_stack (thing output_stack)) input_stack)
;           (apply_many fs (some input_stack))
;         )
;         (= (compose fs) (func input_stack (thing output_stack)))
;       )
;   )
; ))

(assert (not (= (compose (cons (func int2 int1) (cons (func int3 int1) nil))) (some (cons (func (cons int int3) int1) nil)))))

; (declare-const add3_by_composing Option)
; (assert (= add3_by_composing (compose add3)))
; (assert (not (= none add3_by_composing)))

(check-sat)
; (get-model)