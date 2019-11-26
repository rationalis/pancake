(declare-datatypes () (
  (Type
  int
  bool
  (func (in LType) (out LType)))

  (LType nil (cons (head Type) (tail LType)))
))

(define-fun cons_int ((x LType)) LType (cons int x))
(define-fun cons_bool ((x LType)) LType (cons bool x))
(define-const int1 LType (cons_int nil))
(define-const int2 LType (cons_int int1))
(define-const int3 LType (cons_int int2))
(define-const bool1 LType (cons_int nil))

(declare-datatypes (T) (
  (Option (none) (some (thing T)))
))

; Convert non-functions to nullary functions
(define-fun to_func ((f Type)) Type
  (match f (
    ((func I O) f)
    (int (func nil int1))
    (bool (func nil bool1))
  ))
)

(define-fun-rec length ((L LType)) Int
  (match L (
    ((cons H T)
      (+ 1 (length T))
    )
    (nil 0)
  ))
)

(define-fun-rec append ((L1 LType) (L2 LType)) LType
  (match L1 (
    ((cons H T)
     (cons H (append T L2)))
    (nil L2)
  ))
)

(define-fun apply ((f Type) (s LType) (t LType)) Bool
  (= (func s t) f)
)

; proper composition (postfix):
; f: A -> B, g: B -> C, fog: A -> C
(define-fun compose ((f Type) (g Type) (fog Type)) Bool
  (exists ((A LType) (B LType) (C LType))
    (and
      (= (func A B) f)
      (= (func B C) g)
      (= (func A C) fog)
    )
  )
)

; if f: A->B and g: C->D, widen either B or C (and the corresponding A or D) such that
; f/g are unchanged but operate on a larger stack, and can be properly composed, i.e.
; id*,f . id*,g is a valid composition
(define-fun widen ((f Type) (g Type) (f* Type) (g* Type)) Bool
  (exists ((A LType) (B LType) (C LType) (D LType))
    (and
      (= (func A B) f)
      (= (func C D) g)
      (ite (< (length B) (length C))
        (exists ((S LType))
          (and
            (= (append S B) C)
            (= (func (append S A) (append S B)) f*)
            (= g g*)
          )
        )
        (exists ((S LType))
          (and
            (= (append S C) B)
            (= (func (append S C) (append S D)) g*)
            (= f f*)
          )
        )
      )
    )
  )
)

(define-fun gencompose ((f Type) (g Type) (fog Type)) Bool
  (exists ((f* Type) (g* Type))
    (and
      (widen (to_func f) (to_func g) f* g*)
      (compose f* g* fog)
    )
  )
)

(push)
(define-const add Type (func int2 int1))
(declare-const A Type)
(declare-const B Type)
(define-const dup Type (func (cons A nil) (cons A (cons A nil))))
(define-const drop Type (func (cons B nil) nil))

(declare-const I1 Type)
(declare-const I2 Type)

(push)
(assert (= A int))
(assert (= B int))
(assert (gencompose dup add I1)) ; dup + : int -> int
(assert (gencompose I1 drop I2)) ; dup + drop : int -> ()
; (minimize (length (in I2)))
(echo "")
(echo "This should be sat, and determine that I2: int -> ()")
(push)
(check-sat)
(get-value (I2))
(pop)
(echo "")
(echo "This should be unsat, as gencompose should be deterministic.")
(push)
(assert (not (= (func int1 nil) I2)))
(check-sat)
(pop)
(pop)

(push)
(assert (gencompose dup add I1)) ; dup + : int -> int
(assert (gencompose I1 drop I2)) ; dup + drop : int -> ()
(echo "")
(echo "This should be sat, and determine that (A:int) (B:int) (I2: int -> ())")
(check-sat)
(get-value (A))
(get-value (B))
(get-value (I2))
(pop)

(pop)
