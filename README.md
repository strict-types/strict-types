# Strict types AST and typelib implementation

#### Protobufs for functional programming

This is a set of libraries for working with abstract syntax trees and libraries
of [strict types] &ndash; type system made with category theory which ensures
provable properties and bounds for the in-memory and serialized type
representation.

Strict types is a formal notation for defining and serializing
[generalized algebraic data types (GADT)][gadt] in a deterministic
and confined way. It is developed with [type theory] in mind.

Strict Types are:
* __schema-based__ (with the schema being strict encoding notation),
* __semantic__, i.e. defines types not just as they are layed out in memory,
  but also depending on their meaning,
* __deterministic__, i.e. produces the same result for a given type,
* __portabile__, i.e. can run on ahy hardware architecture and OS, including
  low-performant embedded systems,
* __confined__, i.e. provides guarantees and static analysis on a maximum size
  of the typed data,
* __formally verifiabile__.

To learn more about strict encoding [read the spec](https://strict-types.org).

Strict types works with type definitions. It allows:
- static analysis of data types, like
  * defining semantic type ids;
  * specifying exact memory layout;
  * type equivalence in terms of semantics and memory layout;
  * size of serialized data
- composing types into type libraries;
- versioning type libraries basing on the semantic types;

The library allows to generate & compile strict type libraries (STL) from rust 
types implementing `StrictEncoding` trait -- and ensures that the 
deserialization with `StrictDecode` follows the same memory and semantic layout.


## Strict Types Library

The library is able to reflect on itself, producing replica of its rust data
types in strict types system.

Strict types library id:
`stl:G5sL7FHaUo1oBPZ8CXFzqDA7vE3cUzruPMvUbpnBMh3A#biology-laser-popcorn`

Import this lib by putting in the file header
`typelib G5sL7FHaUo1oBPZ8CXFzqDA7vE3cUzruPMvUbpnBMh3A#biology-laser-popcorn`

Source code (you can save it to 
`biology-laser-popcorn.G5sL7FHaUo1oBPZ8CXFzqDA7vE3cUzruPMvUbpnBMh3A.sty` file)

```Haskell
namespace StrictTypes

{-
  Id: G5sL7FHaUo1oBPZ8CXFzqDA7vE3cUzruPMvUbpnBMh3A
  Name: StrictTypes
  Description: Library implementing strict types abstract syntax tree
  Author: Dr Maxim Orlovsky <orlovsky@ubideco.org>
  Copyright: (C) UBIDECO Institute, 2022-2023, Switzerland. All rights reserved.
  License: Apache-2.0
  Checksum: biology-laser-popcorn
-}

-- no dependencies

data BuildFragment    :: ident (Ident) | digits (Ident)
data Dependency       :: id TypeLibId, name LibName, ver SemVer
data EnumVariants     :: ({Variant ^ 1..0xff})
data FieldName        :: (Ident)
data Field_InlineRef  :: name FieldName, ty InlineRef
data Field_InlineRef1 :: name FieldName, ty InlineRef1
data Field_InlineRef2 :: name FieldName, ty InlineRef2
data Field_KeyTy      :: name FieldName, ty KeyTy
data Field_LibRef     :: name FieldName, ty LibRef
data Ident            :: (U8)
data InlineRef        :: inline (Ty_InlineRef1) | named (TypeName, SemId) | extern (TypeName, LibName, SemId)
data InlineRef1       :: inline (Ty_InlineRef2) | named (TypeName, SemId) | extern (TypeName, LibName, SemId)
data InlineRef2       :: inline (Ty_KeyTy) | named (TypeName, SemId) | extern (TypeName, LibName, SemId)
data KeyTy            :: primitive (Primitive) | enum:3 (EnumVariants) | array:7 (U16) | unicode:16 (Sizing) | ascii (Sizing) | bytes (Sizing)
data LibName          :: (Ident)
data LibRef           :: inline (Ty_InlineRef) | named (TypeName, SemId) | extern (TypeName, LibName, SemId)
data LibType          :: name TypeName, ty Ty_LibRef
data NamedFields_InlineRef :: ([Field_InlineRef ^ 1..0xff])
data NamedFields_InlineRef1 :: ([Field_InlineRef1 ^ 1..0xff])
data NamedFields_InlineRef2 :: ([Field_InlineRef2 ^ 1..0xff])
data NamedFields_KeyTy :: ([Field_KeyTy ^ 1..0xff])
data NamedFields_LibRef :: ([Field_LibRef ^ 1..0xff])
data PreFragment      :: ident (Ident) | digits (U128)
data Primitive        :: (U8)
data SemId            :: ([U8 ^ 32])
data SemVer           :: major U16, minor U16, patch U16, pre [PreFragment ^ ..255], build [BuildFragment ^ ..255]
data Sizing           :: min U16, max U16
data Ty_InlineRef     :: primitive (Primitive) | unicode () | enum:3 (EnumVariants) | union (UnionVariants_InlineRef) | tuple (UnnamedFields_InlineRef) | struct (NamedFields_InlineRef) | array (InlineRef, U16) | list (InlineRef, Sizing) | set (InlineRef, Sizing) | map (KeyTy, InlineRef, Sizing)
data Ty_InlineRef1    :: primitive (Primitive) | unicode () | enum:3 (EnumVariants) | union (UnionVariants_InlineRef1) | tuple (UnnamedFields_InlineRef1) | struct (NamedFields_InlineRef1) | array (InlineRef1, U16) | list (InlineRef1, Sizing) | set (InlineRef1, Sizing) | map (KeyTy, InlineRef1, Sizing)
data Ty_InlineRef2    :: primitive (Primitive) | unicode () | enum:3 (EnumVariants) | union (UnionVariants_InlineRef2) | tuple (UnnamedFields_InlineRef2) | struct (NamedFields_InlineRef2) | array (InlineRef2, U16) | list (InlineRef2, Sizing) | set (InlineRef2, Sizing) | map (KeyTy, InlineRef2, Sizing)
data Ty_KeyTy         :: primitive (Primitive) | unicode () | enum:3 (EnumVariants) | union (UnionVariants_KeyTy) | tuple (UnnamedFields_KeyTy) | struct (NamedFields_KeyTy) | array (KeyTy, U16) | list (KeyTy, Sizing) | set (KeyTy, Sizing) | map (KeyTy, KeyTy, Sizing)
data Ty_LibRef        :: primitive (Primitive) | unicode () | enum:3 (EnumVariants) | union (UnionVariants_LibRef) | tuple (UnnamedFields_LibRef) | struct (NamedFields_LibRef) | array (LibRef, U16) | list (LibRef, Sizing) | set (LibRef, Sizing) | map (KeyTy, LibRef, Sizing)
data TypeLib          :: name LibName, dependencies {U8 -> ^ ..255 Dependency}, types {U8 -> ^ 1.. LibType}
data TypeLibId        :: ([U8 ^ 32])
data TypeName         :: (Ident)
data UnionVariants_InlineRef :: ({U8 -> ^ ..255 VariantInfo_InlineRef})
data UnionVariants_InlineRef1 :: ({U8 -> ^ ..255 VariantInfo_InlineRef1})
data UnionVariants_InlineRef2 :: ({U8 -> ^ ..255 VariantInfo_InlineRef2})
data UnionVariants_KeyTy :: ({U8 -> ^ ..255 VariantInfo_KeyTy})
data UnionVariants_LibRef :: ({U8 -> ^ ..255 VariantInfo_LibRef})
data UnnamedFields_InlineRef :: ([InlineRef ^ 1..0xff])
data UnnamedFields_InlineRef1 :: ([InlineRef1 ^ 1..0xff])
data UnnamedFields_InlineRef2 :: ([InlineRef2 ^ 1..0xff])
data UnnamedFields_KeyTy :: ([KeyTy ^ 1..0xff])
data UnnamedFields_LibRef :: ([LibRef ^ 1..0xff])
data Variant          :: name FieldName, tag U8
data VariantInfo_InlineRef :: name FieldName, ty InlineRef
data VariantInfo_InlineRef1 :: name FieldName, ty InlineRef1
data VariantInfo_InlineRef2 :: name FieldName, ty InlineRef2
data VariantInfo_KeyTy :: name FieldName, ty KeyTy
data VariantInfo_LibRef :: name FieldName, ty LibRef
```

Base-64 encoded compiled library (you can save it into
`biology-laser-popcorn.G5sL7FHaUo1oBPZ8CXFzqDA7vE3cUzruPMvUbpnBMh3A.stl`):

```
----- BEGIN STRICT TYPE LIB -----
Id: G5sL7FHaUo1oBPZ8CXFzqDA7vE3cUzruPMvUbpnBMh3A
Name: StrictTypes
Description: Library implementing strict types abstract syntax tree
Author: Dr Maxim Orlovsky <orlovsky@ubideco.org>
Copyright: (C) UBIDECO Institute, 2022-2023, Switzerland. All rights reserved.
License: Apache-2.0
Checksum: biology-laser-popcorn

C1N0cmljdFR5cGVzADMAAA1CdWlsZEZyYWdtZW50DUJ1aWxkRnJhZ21lbnQEAgAFaWRlbnQABQEBBUlk
ZW50CdI22uW0c9If4NPdnO5fDyf8JNoFB+hGwecbAo1JVYQBBmRpZ2l0cwAFAQEFSWRlbnQJ0jba5bRz
0h/g092c7l8PJ/wk2gUH6EbB5xsCjUlVhApEZXBlbmRlbmN5CkRlcGVuZGVuY3kGAwJpZAEJVHlwZUxp
Yklkn+SImkykGwxPBBIcxjRcxW0+8qa9Josge6hOPVikSGsEbmFtZQEHTGliTmFtZeMkuXJ8NtAYXA7C
OyuYCihW1f7QH5x638vI4Ir6F6rBA3ZlcgEGU2VtVmVyPsV98RGI1NMR5cyzAZ53eGFh2bXsMbzY8kz6
cJIa2O0MRW51bVZhcmlhbnRzDEVudW1WYXJpYW50cwUBAAkBB1ZhcmlhbnQdhzCzEG7Prm6udMjYjRC8
+23967J/Te3xenEIxdXkVwEA/wAJRmllbGROYW1lCUZpZWxkTmFtZQUBAQVJZGVudAnSNtrltHPSH+DT
3ZzuXw8n/CTaBQfoRsHnGwKNSVWED0ZpZWxkX0lubGluZVJlZg9GaWVsZF9JbmxpbmVSZWYGAgRuYW1l
AQlGaWVsZE5hbWW8Vt2BDia7b5GB9tJQNcWOfQWK/vFpZy0on4zR8j9oegJ0eQEJSW5saW5lUmVm6tvV
42bVzbQxhTvOslQ2X9bce+MAk4cFPO4sHfMVD7oQRmllbGRfSW5saW5lUmVmMRBGaWVsZF9JbmxpbmVS
ZWYxBgIEbmFtZQEJRmllbGROYW1lvFbdgQ4mu2+RgfbSUDXFjn0Fiv7xaWctKJ+M0fI/aHoCdHkBCklu
bGluZVJlZjG9hHrUIX2JiNEdfCnnzwkFAzcZSJOmSFJGBY4OJmr2iRBGaWVsZF9JbmxpbmVSZWYyEEZp
ZWxkX0lubGluZVJlZjIGAgRuYW1lAQlGaWVsZE5hbWW8Vt2BDia7b5GB9tJQNcWOfQWK/vFpZy0on4zR
8j9oegJ0eQEKSW5saW5lUmVmMlZUxQ9nyWQOY7e83ErehiMPFNJC4ZV1tU3IAHCs6bOKC0ZpZWxkX0tl
eVR5C0ZpZWxkX0tleVR5BgIEbmFtZQEJRmllbGROYW1lvFbdgQ4mu2+RgfbSUDXFjn0Fiv7xaWctKJ+M
0fI/aHoCdHkBBUtleVR5QrYSbC7jJiGszKH9EFkNSkrGEdaGVA4Xs042DHTi3TMMRmllbGRfTGliUmVm
DEZpZWxkX0xpYlJlZgYCBG5hbWUBCUZpZWxkTmFtZbxW3YEOJrtvkYH20lA1xY59BYr+8WlnLSifjNHy
P2h6AnR5AQZMaWJSZWYIs6MogE9vroIQv/OPLU5nNMcNHznJ42ymIoD4ztrP0gVJZGVudAVJZGVudAUB
AAABCUlubGluZVJlZglJbmxpbmVSZWYEAwAGaW5saW5lAAUBAQ1UeV9JbmxpbmVSZWYxp1wr6k8Eh4tg
HUIjjvf8JdIG3nqVTSFXKCrPIBULVocBBW5hbWVkAAUCAQhUeXBlTmFtZe43VN8wq2iJmnqZdaIWjvLP
t9fOGwNyBBqfoPKvXDBqAQVTZW1JZE6Y8obHPHirJZhZqdQi7wFL+CrrwdHaujVNtcWCCEb/AgZleHRl
cm4ABQMBCFR5cGVOYW1l7jdU3zCraImaepl1ohaO8s+3184bA3IEGp+g8q9cMGoBB0xpYk5hbWXjJLly
fDbQGFwOwjsrmAooVtX+0B+cet/LyOCK+heqwQEFU2VtSWROmPKGxzx4qyWYWanUIu8BS/gq68HR2ro1
TbXFgghG/wpJbmxpbmVSZWYxCklubGluZVJlZjEEAwAGaW5saW5lAAUBAQ1UeV9JbmxpbmVSZWYywVEb
TbZQxsgnp3pXzXGrAvO8Vfk/riBOdM9jIbGdLoYBBW5hbWVkAAUCAQhUeXBlTmFtZe43VN8wq2iJmnqZ
daIWjvLPt9fOGwNyBBqfoPKvXDBqAQVTZW1JZE6Y8obHPHirJZhZqdQi7wFL+CrrwdHaujVNtcWCCEb/
AgZleHRlcm4ABQMBCFR5cGVOYW1l7jdU3zCraImaepl1ohaO8s+3184bA3IEGp+g8q9cMGoBB0xpYk5h
bWXjJLlyfDbQGFwOwjsrmAooVtX+0B+cet/LyOCK+heqwQEFU2VtSWROmPKGxzx4qyWYWanUIu8BS/gq
68HR2ro1TbXFgghG/wpJbmxpbmVSZWYyCklubGluZVJlZjIEAwAGaW5saW5lAAUBAQhUeV9LZXlUeZ0r
5q195fmrrKOJI78rXyb7RiIFy9fCrsnSRYVQH3tDAQVuYW1lZAAFAgEIVHlwZU5hbWXuN1TfMKtoiZp6
mXWiFo7yz7fXzhsDcgQan6Dyr1wwagEFU2VtSWROmPKGxzx4qyWYWanUIu8BS/gq68HR2ro1TbXFgghG
/wIGZXh0ZXJuAAUDAQhUeXBlTmFtZe43VN8wq2iJmnqZdaIWjvLPt9fOGwNyBBqfoPKvXDBqAQdMaWJO
YW1l4yS5cnw20BhcDsI7K5gKKFbV/tAfnHrfy8jgivoXqsEBBVNlbUlkTpjyhsc8eKslmFmp1CLvAUv4
KuvB0dq6NU21xYIIRv8FS2V5VHkFS2V5VHkEBgAJcHJpbWl0aXZlAAUBAQlQcmltaXRpdmUrxVyrp1/U
V7gbwapXRgFljhZe4GFjR0heVesPC/9vtwMEZW51bQAFAQEMRW51bVZhcmlhbnRzs2/otIwq7FFKfG+G
KHNP2Pspo9/xoqJ4q84katrFyxcHBWFycmF5AAUBAAACEAd1bmljb2RlAAUBAQZTaXppbmfCyKDT4vdF
J25EV2o+mQ1eBOBMO9PRPdf88ymRYoFUexEFYXNjaWkABQEBBlNpemluZ8LIoNPi90UnbkRXaj6ZDV4E
4Ew709E91/zzKZFigVR7EgVieXRlcwAFAQEGU2l6aW5nwsig0+L3RSduRFdqPpkNXgTgTDvT0T3X/PMp
kWKBVHsHTGliTmFtZQdMaWJOYW1lBQEBBUlkZW50CdI22uW0c9If4NPdnO5fDyf8JNoFB+hGwecbAo1J
VYQGTGliUmVmBkxpYlJlZgQDAAZpbmxpbmUABQEBDFR5X0lubGluZVJlZnuXRWOnVf+E1moP6cBRS+JM
8cgVhfKzhF4LIuNQbU5VAQVuYW1lZAAFAgEIVHlwZU5hbWXuN1TfMKtoiZp6mXWiFo7yz7fXzhsDcgQa
n6Dyr1wwagEFU2VtSWROmPKGxzx4qyWYWanUIu8BS/gq68HR2ro1TbXFgghG/wIGZXh0ZXJuAAUDAQhU
eXBlTmFtZe43VN8wq2iJmnqZdaIWjvLPt9fOGwNyBBqfoPKvXDBqAQdMaWJOYW1l4yS5cnw20BhcDsI7
K5gKKFbV/tAfnHrfy8jgivoXqsEBBVNlbUlkTpjyhsc8eKslmFmp1CLvAUv4KuvB0dq6NU21xYIIRv8H
TGliVHlwZQdMaWJUeXBlBgIEbmFtZQEIVHlwZU5hbWXuN1TfMKtoiZp6mXWiFo7yz7fXzhsDcgQan6Dy
r1wwagJ0eQEJVHlfTGliUmVm2ynl+oFTmxeAJ14BK9uqxwsQ4nnkXYzPwGjxbo4VJaEVTmFtZWRGaWVs
ZHNfSW5saW5lUmVmFU5hbWVkRmllbGRzX0lubGluZVJlZgUBAAgBD0ZpZWxkX0lubGluZVJlZqJlvq0B
aG0SmBF1luD4pXqbQayghUcOVk8H0Fq6aROdAQD/ABZOYW1lZEZpZWxkc19JbmxpbmVSZWYxFk5hbWVk
RmllbGRzX0lubGluZVJlZjEFAQAIARBGaWVsZF9JbmxpbmVSZWYxtpz1lSNVbeNg9k1FI4L/89wJRkA1
u1TStQb+g+qtT9EBAP8AFk5hbWVkRmllbGRzX0lubGluZVJlZjIWTmFtZWRGaWVsZHNfSW5saW5lUmVm
MgUBAAgBEEZpZWxkX0lubGluZVJlZjKSEQ7EYLq4YQrX1CuuNUL0HILRRrgBQCNjz7SC4cZ3SAEA/wAR
TmFtZWRGaWVsZHNfS2V5VHkRTmFtZWRGaWVsZHNfS2V5VHkFAQAIAQtGaWVsZF9LZXlUeX04irtTmANl
GbgN3kpTDg9ap4X5P4I90+R5CSflcXX7AQD/ABJOYW1lZEZpZWxkc19MaWJSZWYSTmFtZWRGaWVsZHNf
TGliUmVmBQEACAEMRmllbGRfTGliUmVmEqZGNUvRatWExzNWmYggzaEEO89xDOfNb7XIRAduRloBAP8A
C1ByZUZyYWdtZW50C1ByZUZyYWdtZW50BAIABWlkZW50AAUBAQVJZGVudAnSNtrltHPSH+DT3ZzuXw8n
/CTaBQfoRsHnGwKNSVWEAQZkaWdpdHMABQEAABAJUHJpbWl0aXZlCVByaW1pdGl2ZQUBAAABBVNlbUlk
BVNlbUlkBQEABwAAASAABlNlbVZlcgZTZW1WZXIGBQVtYWpvcgAAAgVtaW5vcgAAAgVwYXRjaAAAAgNw
cmUACAELUHJlRnJhZ21lbnRBr6J7XH65StWKQEV1trzwATZJy4NjWziYLknmKLu6oAAA/wAFYnVpbGQA
CAENQnVpbGRGcmFnbWVudOPoQaM/bpjv76uU56RVlvUcR7mdU8GdKGSN2JjnRNOqAAD/AAZTaXppbmcG
U2l6aW5nBgIDbWluAAACA21heAAAAgxUeV9JbmxpbmVSZWYMVHlfSW5saW5lUmVmBAoACXByaW1pdGl2
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
CgABAQpEZXBlbmRlbmN5EjOaYyMF5ykW9NH2P5j/U2MH8sDJ3+jgOqTwLCA4eeIAAP8ABXR5cGVzAAoA
AQEHTGliVHlwZTuDa0FtdPYxlRGc0nBNeBQpcKnAi0w+oN/w3PWvmvpEAQD//wlUeXBlTGliSWQJVHlw
ZUxpYklkBQEABwAAASAACFR5cGVOYW1lCFR5cGVOYW1lBQEBBUlkZW50CdI22uW0c9If4NPdnO5fDyf8
JNoFB+hGwecbAo1JVYQXVW5pb25WYXJpYW50c19JbmxpbmVSZWYXVW5pb25WYXJpYW50c19JbmxpbmVS
ZWYFAQAKAAEBFVZhcmlhbnRJbmZvX0lubGluZVJlZg4UpsmKKI24qFfK4CvLBvHqKpJSE7takSbIrRok
jqvoAAD/ABhVbmlvblZhcmlhbnRzX0lubGluZVJlZjEYVW5pb25WYXJpYW50c19JbmxpbmVSZWYxBQEA
CgABARZWYXJpYW50SW5mb19JbmxpbmVSZWYxf+mQPU3KYDPgRrqAnn+IYIDTvQeVRY3Dn+mkUV8pUksA
AP8AGFVuaW9uVmFyaWFudHNfSW5saW5lUmVmMhhVbmlvblZhcmlhbnRzX0lubGluZVJlZjIFAQAKAAEB
FlZhcmlhbnRJbmZvX0lubGluZVJlZjIQ/fvNIknULCm6y4VwFsIyOAdlY6chLrpJNYLeUKpL1AAA/wAT
VW5pb25WYXJpYW50c19LZXlUeRNVbmlvblZhcmlhbnRzX0tleVR5BQEACgABARFWYXJpYW50SW5mb19L
ZXlUeVrSVMd3rdv1MABr94CnFia/0Dw9KwymVUtw9wIP1RdTAAD/ABRVbmlvblZhcmlhbnRzX0xpYlJl
ZhRVbmlvblZhcmlhbnRzX0xpYlJlZgUBAAoAAQESVmFyaWFudEluZm9fTGliUmVmMCKvQXXYEPKwjszl
KDGC6wb9AuAvVKmfrA3eP4jS7ggAAP8AF1VubmFtZWRGaWVsZHNfSW5saW5lUmVmF1VubmFtZWRGaWVs
ZHNfSW5saW5lUmVmBQEACAEJSW5saW5lUmVm6tvV42bVzbQxhTvOslQ2X9bce+MAk4cFPO4sHfMVD7oB
AP8AGFVubmFtZWRGaWVsZHNfSW5saW5lUmVmMRhVbm5hbWVkRmllbGRzX0lubGluZVJlZjEFAQAIAQpJ
bmxpbmVSZWYxvYR61CF9iYjRHXwp588JBQM3GUiTpkhSRgWODiZq9okBAP8AGFVubmFtZWRGaWVsZHNf
SW5saW5lUmVmMhhVbm5hbWVkRmllbGRzX0lubGluZVJlZjIFAQAIAQpJbmxpbmVSZWYyVlTFD2fJZA5j
t7zcSt6GIw8U0kLhlXW1TcgAcKzps4oBAP8AE1VubmFtZWRGaWVsZHNfS2V5VHkTVW5uYW1lZEZpZWxk
c19LZXlUeQUBAAgBBUtleVR5QrYSbC7jJiGszKH9EFkNSkrGEdaGVA4Xs042DHTi3TMBAP8AFFVubmFt
ZWRGaWVsZHNfTGliUmVmFFVubmFtZWRGaWVsZHNfTGliUmVmBQEACAEGTGliUmVmCLOjKIBPb66CEL/z
jy1OZzTHDR85yeNspiKA+M7az9IBAP8AB1ZhcmlhbnQHVmFyaWFudAYCBG5hbWUBCUZpZWxkTmFtZbxW
3YEOJrtvkYH20lA1xY59BYr+8WlnLSifjNHyP2h6A3RhZwAAARVWYXJpYW50SW5mb19JbmxpbmVSZWYV
VmFyaWFudEluZm9fSW5saW5lUmVmBgIEbmFtZQEJRmllbGROYW1lvFbdgQ4mu2+RgfbSUDXFjn0Fiv7x
aWctKJ+M0fI/aHoCdHkBCUlubGluZVJlZurb1eNm1c20MYU7zrJUNl/W3HvjAJOHBTzuLB3zFQ+6FlZh
cmlhbnRJbmZvX0lubGluZVJlZjEWVmFyaWFudEluZm9fSW5saW5lUmVmMQYCBG5hbWUBCUZpZWxkTmFt
ZbxW3YEOJrtvkYH20lA1xY59BYr+8WlnLSifjNHyP2h6AnR5AQpJbmxpbmVSZWYxvYR61CF9iYjRHXwp
588JBQM3GUiTpkhSRgWODiZq9okWVmFyaWFudEluZm9fSW5saW5lUmVmMhZWYXJpYW50SW5mb19Jbmxp
bmVSZWYyBgIEbmFtZQEJRmllbGROYW1lvFbdgQ4mu2+RgfbSUDXFjn0Fiv7xaWctKJ+M0fI/aHoCdHkB
CklubGluZVJlZjJWVMUPZ8lkDmO3vNxK3oYjDxTSQuGVdbVNyABwrOmzihFWYXJpYW50SW5mb19LZXlU
eRFWYXJpYW50SW5mb19LZXlUeQYCBG5hbWUBCUZpZWxkTmFtZbxW3YEOJrtvkYH20lA1xY59BYr+8Wln
LSifjNHyP2h6AnR5AQVLZXlUeUK2Emwu4yYhrMyh/RBZDUpKxhHWhlQOF7NONgx04t0zElZhcmlhbnRJ
bmZvX0xpYlJlZhJWYXJpYW50SW5mb19MaWJSZWYGAgRuYW1lAQlGaWVsZE5hbWW8Vt2BDia7b5GB9tJQ
NcWOfQWK/vFpZy0on4zR8j9oegJ0eQEGTGliUmVmCLOjKIBPb66CEL/zjy1OZzTHDR85yeNspiKA+M7a
z9I=

----- END STRICT TYPE LIB -----
```

## Contributing

[CONTRIBUTING.md](../CONTRIBUTING.md)

## License

The libraries are distributed on the terms of [Apache 2.0 license](LICENSE).

[strict types]: https://strict-types.org
[gadt]: https://en.wikipedia.org/wiki/Algebraic_data_type
[type theory]: https://en.wikipedia.org/wiki/Type_theory
