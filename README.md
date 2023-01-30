# Rust implementation of strict encoding schema (STENS)

Strict encoding is a deterministic schema-base binary serialization format 
for algebraic types (ADT) which provides automatic strong type checking. It is
used in consensus protocols, networking, AluVM and long-term data storage. 

This library provides primitives for describing strict encoding schemata, 
validating and parsing structured data using the schema and for on-the-fly
checking of algebraic data type serialziation/deserialization.

To learn more about strict encoding [read the spec](https://www.strict-encoding.org).

Strict encoding schema works with type definitions. It allows:
- static analysis of data types, like
  * defining semantic type ids;
  * specifying exact memory layout;
  * type equivalence in terms of semantics and memory layout;
  * size of serialized data
- composing types into type libraries;
- versioning type libraries basing on the semantic types;

Current rust implementation additionally allows to build type libraries out of
rust data types which implement `StrictEncoding` trait -- and ensures that the
deserialization with `StrictDecode` follows the same memory and semantic layout.

## Current version

Sty library id:
`stl:9xg6UrhKe5yWf4oqPpYgXVkZPGRmaA8Q6ZAmu84rDFNm#popcorn-yellow-moses`

```Haskell
{-
-- Import this lib by putting in the file header
-- import popcorn_yellow_moses_9xg6UrhKe5yWf4oqPpYgXVkZPGRmaA8Q6ZAmu84rDFNm
-}
namespace StEn -- popcorn_yellow_moses_9xg6UrhKe5yWf4oqPpYgXVkZPGRmaA8Q6ZAmu84rDFNm.stl

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
data KeyTy            :: primitive (Primitive) | unicode (Sizing) | ascii (Sizing) | enum (EnumVariants) | array:7 (U16) | bytes (Sizing)
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
data Ty_InlineRef     :: primitive (Primitive) | unicode () | enum:3 (EnumVariants) | union (UnionVariants_InlineRef) | struct (NamedFields_InlineRef) | tuple (UnnamedFields_InlineRef) | array (InlineRef, U16) | list (InlineRef, Sizing) | set (InlineRef, Sizing) | map (KeyTy, InlineRef, Sizing)
data Ty_InlineRef1    :: primitive (Primitive) | unicode () | enum:3 (EnumVariants) | union (UnionVariants_InlineRef1) | struct (NamedFields_InlineRef1) | tuple (UnnamedFields_InlineRef1) | array (InlineRef1, U16) | list (InlineRef1, Sizing) | set (InlineRef1, Sizing) | map (KeyTy, InlineRef1, Sizing)
data Ty_InlineRef2    :: primitive (Primitive) | unicode () | enum:3 (EnumVariants) | union (UnionVariants_InlineRef2) | struct (NamedFields_InlineRef2) | tuple (UnnamedFields_InlineRef2) | array (InlineRef2, U16) | list (InlineRef2, Sizing) | set (InlineRef2, Sizing) | map (KeyTy, InlineRef2, Sizing)
data Ty_KeyTy         :: primitive (Primitive) | unicode () | enum:3 (EnumVariants) | union (UnionVariants_KeyTy) | struct (NamedFields_KeyTy) | tuple (UnnamedFields_KeyTy) | array (KeyTy, U16) | list (KeyTy, Sizing) | set (KeyTy, Sizing) | map (KeyTy, KeyTy, Sizing)
data Ty_LibRef        :: primitive (Primitive) | unicode () | enum:3 (EnumVariants) | union (UnionVariants_LibRef) | struct (NamedFields_LibRef) | tuple (UnnamedFields_LibRef) | array (LibRef, U16) | list (LibRef, Sizing) | set (LibRef, Sizing) | map (KeyTy, LibRef, Sizing)
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

Encoded library:
```
----- BEGIN STRICT TYPE LIB -----
Id: 9xg6UrhKe5yWf4oqPpYgXVkZPGRmaA8Q6ZAmu84rDFNm
Checksum: popcorn-yellow-moses

BFN0RW4AMwAADUJ1aWxkRnJhZ21lbnQNQnVpbGRGcmFnbWVudAQCAAVpZGVudAAGAQEFSWRlbnQJ0jba
5bRz0h/g092c7l8PJ/wk2gUH6EbB5xsCjUlVhAEGZGlnaXRzAAYBAQVJZGVudAnSNtrltHPSH+DT3Zzu
Xw8n/CTaBQfoRsHnGwKNSVWECkRlcGVuZGVuY3kKRGVwZW5kZW5jeQUDAmlkAQlUeXBlTGliSWSf5Iia
TKQbDE8EEhzGNFzFbT7ypr0miyB7qE49WKRIawRuYW1lAQdMaWJOYW1l4yS5cnw20BhcDsI7K5gKKFbV
/tAfnHrfy8jgivoXqsEDdmVyAQZTZW1WZXI+xX3xEYjU0xHlzLMBnnd4YWHZtewxvNjyTPpwkhrY7QxF
bnVtVmFyaWFudHMMRW51bVZhcmlhbnRzBgEACQEHVmFyaWFudB2HMLMQbs+ubq50yNiNELz7bf3rsn9N
7fF6cQjF1eRXAQD/AAlGaWVsZE5hbWUJRmllbGROYW1lBgEBBUlkZW50CdI22uW0c9If4NPdnO5fDyf8
JNoFB+hGwecbAo1JVYQPRmllbGRfSW5saW5lUmVmD0ZpZWxkX0lubGluZVJlZgUCBG5hbWUBCUZpZWxk
TmFtZbxW3YEOJrtvkYH20lA1xY59BYr+8WlnLSifjNHyP2h6AnR5AQlJbmxpbmVSZWathRk7lyPzxrQd
w5Z5oC2avxWWjQk4wXxR6BwvPmYAmxBGaWVsZF9JbmxpbmVSZWYxEEZpZWxkX0lubGluZVJlZjEFAgRu
YW1lAQlGaWVsZE5hbWW8Vt2BDia7b5GB9tJQNcWOfQWK/vFpZy0on4zR8j9oegJ0eQEKSW5saW5lUmVm
McCVh0NAczDxhbDi+ubcwFhZFLbp9zw5qZBKh7N4zFyDEEZpZWxkX0lubGluZVJlZjIQRmllbGRfSW5s
aW5lUmVmMgUCBG5hbWUBCUZpZWxkTmFtZbxW3YEOJrtvkYH20lA1xY59BYr+8WlnLSifjNHyP2h6AnR5
AQpJbmxpbmVSZWYyy0ZF+vl+FjNHVC0TYlfQU8X9m9OfN1Gg0ikBvCtVZEQLRmllbGRfS2V5VHkLRmll
bGRfS2V5VHkFAgRuYW1lAQlGaWVsZE5hbWW8Vt2BDia7b5GB9tJQNcWOfQWK/vFpZy0on4zR8j9oegJ0
eQEFS2V5VHkhoor3IhSPo0cc52I42sgNrpeaUadXFtbaSyKLge4dkwxGaWVsZF9MaWJSZWYMRmllbGRf
TGliUmVmBQIEbmFtZQEJRmllbGROYW1lvFbdgQ4mu2+RgfbSUDXFjn0Fiv7xaWctKJ+M0fI/aHoCdHkB
BkxpYlJlZhLTacozGnMcJ0m1Rk7JPIPftsNKPzkZ/iXNqXWZdhF9BUlkZW50BUlkZW50BgEAAAEJSW5s
aW5lUmVmCUlubGluZVJlZgQDAAZpbmxpbmUABgEBDVR5X0lubGluZVJlZjGCuzV+1NxBlrHigmE82I/q
XqX+wFXuT7B7XvXHuqd9fQEFbmFtZWQABgIBCFR5cGVOYW1l7jdU3zCraImaepl1ohaO8s+3184bA3IE
Gp+g8q9cMGoBBVNlbUlkTpjyhsc8eKslmFmp1CLvAUv4KuvB0dq6NU21xYIIRv8CBmV4dGVybgAGAwEI
VHlwZU5hbWXuN1TfMKtoiZp6mXWiFo7yz7fXzhsDcgQan6Dyr1wwagEHTGliTmFtZeMkuXJ8NtAYXA7C
OyuYCihW1f7QH5x638vI4Ir6F6rBAQVTZW1JZE6Y8obHPHirJZhZqdQi7wFL+CrrwdHaujVNtcWCCEb/
CklubGluZVJlZjEKSW5saW5lUmVmMQQDAAZpbmxpbmUABgEBDVR5X0lubGluZVJlZjKrxBagANUD7QFy
RRUvZZXxl7fDJ9pW4ug4/0h8B1JYawEFbmFtZWQABgIBCFR5cGVOYW1l7jdU3zCraImaepl1ohaO8s+3
184bA3IEGp+g8q9cMGoBBVNlbUlkTpjyhsc8eKslmFmp1CLvAUv4KuvB0dq6NU21xYIIRv8CBmV4dGVy
bgAGAwEIVHlwZU5hbWXuN1TfMKtoiZp6mXWiFo7yz7fXzhsDcgQan6Dyr1wwagEHTGliTmFtZeMkuXJ8
NtAYXA7COyuYCihW1f7QH5x638vI4Ir6F6rBAQVTZW1JZE6Y8obHPHirJZhZqdQi7wFL+CrrwdHaujVN
tcWCCEb/CklubGluZVJlZjIKSW5saW5lUmVmMgQDAAZpbmxpbmUABgEBCFR5X0tleVR5F57lYQXfMWET
mtvsZbTIl0vHktjfTz0WIXTl7XucfbEBBW5hbWVkAAYCAQhUeXBlTmFtZe43VN8wq2iJmnqZdaIWjvLP
t9fOGwNyBBqfoPKvXDBqAQVTZW1JZE6Y8obHPHirJZhZqdQi7wFL+CrrwdHaujVNtcWCCEb/AgZleHRl
cm4ABgMBCFR5cGVOYW1l7jdU3zCraImaepl1ohaO8s+3184bA3IEGp+g8q9cMGoBB0xpYk5hbWXjJLly
fDbQGFwOwjsrmAooVtX+0B+cet/LyOCK+heqwQEFU2VtSWROmPKGxzx4qyWYWanUIu8BS/gq68HR2ro1
TbXFgghG/wVLZXlUeQVLZXlUeQQGAAlwcmltaXRpdmUABgEBCVByaW1pdGl2ZSvFXKunX9RXuBvBqldG
AWWOFl7gYWNHSF5V6w8L/2+3AQd1bmljb2RlAAYBAQZTaXppbmfCyKDT4vdFJ25EV2o+mQ1eBOBMO9PR
Pdf88ymRYoFUewIFYXNjaWkABgEBBlNpemluZ8LIoNPi90UnbkRXaj6ZDV4E4Ew709E91/zzKZFigVR7
AwRlbnVtAAYBAQxFbnVtVmFyaWFudHOzb+i0jCrsUUp8b4Yoc0/Y+ymj3/GionirziRq2sXLFwcFYXJy
YXkABgEAAAIIBWJ5dGVzAAYBAQZTaXppbmfCyKDT4vdFJ25EV2o+mQ1eBOBMO9PRPdf88ymRYoFUewdM
aWJOYW1lB0xpYk5hbWUGAQEFSWRlbnQJ0jba5bRz0h/g092c7l8PJ/wk2gUH6EbB5xsCjUlVhAZMaWJS
ZWYGTGliUmVmBAMABmlubGluZQAGAQEMVHlfSW5saW5lUmVmmSkveF/01tGZqZ+JtzOy7A8hchnfpdc+
yaMTkdRNGNgBBW5hbWVkAAYCAQhUeXBlTmFtZe43VN8wq2iJmnqZdaIWjvLPt9fOGwNyBBqfoPKvXDBq
AQVTZW1JZE6Y8obHPHirJZhZqdQi7wFL+CrrwdHaujVNtcWCCEb/AgZleHRlcm4ABgMBCFR5cGVOYW1l
7jdU3zCraImaepl1ohaO8s+3184bA3IEGp+g8q9cMGoBB0xpYk5hbWXjJLlyfDbQGFwOwjsrmAooVtX+
0B+cet/LyOCK+heqwQEFU2VtSWROmPKGxzx4qyWYWanUIu8BS/gq68HR2ro1TbXFgghG/wdMaWJUeXBl
B0xpYlR5cGUFAgRuYW1lAQhUeXBlTmFtZe43VN8wq2iJmnqZdaIWjvLPt9fOGwNyBBqfoPKvXDBqAnR5
AQlUeV9MaWJSZWYEenHw+40fBdVfmC3XK7ExAN7FREQP1X3WODeOKroyTBVOYW1lZEZpZWxkc19Jbmxp
bmVSZWYVTmFtZWRGaWVsZHNfSW5saW5lUmVmBgEACAEPRmllbGRfSW5saW5lUmVmmnjwfOQAOSyTf+oF
2JkgrDcwiMCKdc7/9gLJ0vaq2aoBAP8AFk5hbWVkRmllbGRzX0lubGluZVJlZjEWTmFtZWRGaWVsZHNf
SW5saW5lUmVmMQYBAAgBEEZpZWxkX0lubGluZVJlZjFXQmsGaUHgMqhcpZqhD0w2ebc6HBS17OjOBamx
8rTi2QEA/wAWTmFtZWRGaWVsZHNfSW5saW5lUmVmMhZOYW1lZEZpZWxkc19JbmxpbmVSZWYyBgEACAEQ
RmllbGRfSW5saW5lUmVmMtzT+z8W2EwEJM6FUMVk/s1VD0fobY6iATAfRGqCRxwUAQD/ABFOYW1lZEZp
ZWxkc19LZXlUeRFOYW1lZEZpZWxkc19LZXlUeQYBAAgBC0ZpZWxkX0tleVR54Xm5HjtURHethyFQsbyn
GD4Tyzk/QUuSzv96ADiQMwkBAP8AEk5hbWVkRmllbGRzX0xpYlJlZhJOYW1lZEZpZWxkc19MaWJSZWYG
AQAIAQxGaWVsZF9MaWJSZWa7GB/7kaLHHI6awkGEyJY4E+Ju6XJCt2w9vwl0FaPLzgEA/wALUHJlRnJh
Z21lbnQLUHJlRnJhZ21lbnQEAgAFaWRlbnQABgEBBUlkZW50CdI22uW0c9If4NPdnO5fDyf8JNoFB+hG
wecbAo1JVYQBBmRpZ2l0cwAGAQAAEAlQcmltaXRpdmUJUHJpbWl0aXZlBgEAAAEFU2VtSWQFU2VtSWQG
AQAHAAABIAAGU2VtVmVyBlNlbVZlcgUFBW1ham9yAAACBW1pbm9yAAACBXBhdGNoAAACA3ByZQAIAQtQ
cmVGcmFnbWVudEGvontcfrlK1YpARXW2vPABNknLg2NbOJguSeYou7qgAAD/AAVidWlsZAAIAQ1CdWls
ZEZyYWdtZW504+hBoz9umO/vq5TnpFWW9RxHuZ1TwZ0oZI3YmOdE06oAAP8ABlNpemluZwZTaXppbmcF
AgNtaW4AAAIDbWF4AAACDFR5X0lubGluZVJlZgxUeV9JbmxpbmVSZWYECgAJcHJpbWl0aXZlAAYBAQlQ
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
cGVuZGVuY3kSM5pjIwXnKRb00fY/mP9TYwfywMnf6OA6pPAsIDh54gAA/wAFdHlwZXMACgABAQdMaWJU
eXBl2ZBtknU1A33kfp9pn5zuGLIDFV0F+3nv6vHu3urQPm0BAP//CVR5cGVMaWJJZAlUeXBlTGliSWQG
AQAHAAABIAAIVHlwZU5hbWUIVHlwZU5hbWUGAQEFSWRlbnQJ0jba5bRz0h/g092c7l8PJ/wk2gUH6EbB
5xsCjUlVhBdVbmlvblZhcmlhbnRzX0lubGluZVJlZhdVbmlvblZhcmlhbnRzX0lubGluZVJlZgYBAAoA
AQEVVmFyaWFudEluZm9fSW5saW5lUmVmTwSLUDg2kEGyzO+0U8HcUNjGsQmD/EgEAn4ozWunta0AAP8A
GFVuaW9uVmFyaWFudHNfSW5saW5lUmVmMRhVbmlvblZhcmlhbnRzX0lubGluZVJlZjEGAQAKAAEBFlZh
cmlhbnRJbmZvX0lubGluZVJlZjECjlqXV1Uyv5ixEnykQiRphAUwFb27ZX9JoNFPS4h/KQAA/wAYVW5p
b25WYXJpYW50c19JbmxpbmVSZWYyGFVuaW9uVmFyaWFudHNfSW5saW5lUmVmMgYBAAoAAQEWVmFyaWFu
dEluZm9fSW5saW5lUmVmMgEVmcS098MiLQGyN3KNiDKPPhXWHbarc18mDlM02L8NAAD/ABNVbmlvblZh
cmlhbnRzX0tleVR5E1VuaW9uVmFyaWFudHNfS2V5VHkGAQAKAAEBEVZhcmlhbnRJbmZvX0tleVR5KW2Y
Ur54GIHgzQGz6Li98L6/sDPluqbb6YpatK9VRzgAAP8AFFVuaW9uVmFyaWFudHNfTGliUmVmFFVuaW9u
VmFyaWFudHNfTGliUmVmBgEACgABARJWYXJpYW50SW5mb19MaWJSZWb4PnLahaojh/PDnslenAtKoHxm
cscR25OOC1cO7a3cJwAA/wAXVW5uYW1lZEZpZWxkc19JbmxpbmVSZWYXVW5uYW1lZEZpZWxkc19Jbmxp
bmVSZWYGAQAIAQlJbmxpbmVSZWathRk7lyPzxrQdw5Z5oC2avxWWjQk4wXxR6BwvPmYAmwEA/wAYVW5u
YW1lZEZpZWxkc19JbmxpbmVSZWYxGFVubmFtZWRGaWVsZHNfSW5saW5lUmVmMQYBAAgBCklubGluZVJl
ZjHAlYdDQHMw8YWw4vrm3MBYWRS26fc8OamQSoezeMxcgwEA/wAYVW5uYW1lZEZpZWxkc19JbmxpbmVS
ZWYyGFVubmFtZWRGaWVsZHNfSW5saW5lUmVmMgYBAAgBCklubGluZVJlZjLLRkX6+X4WM0dULRNiV9BT
xf2b0583UaDSKQG8K1VkRAEA/wATVW5uYW1lZEZpZWxkc19LZXlUeRNVbm5hbWVkRmllbGRzX0tleVR5
BgEACAEFS2V5VHkhoor3IhSPo0cc52I42sgNrpeaUadXFtbaSyKLge4dkwEA/wAUVW5uYW1lZEZpZWxk
c19MaWJSZWYUVW5uYW1lZEZpZWxkc19MaWJSZWYGAQAIAQZMaWJSZWYS02nKMxpzHCdJtUZOyTyD37bD
Sj85Gf4lzal1mXYRfQEA/wAHVmFyaWFudAdWYXJpYW50BQIEbmFtZQEJRmllbGROYW1lvFbdgQ4mu2+R
gfbSUDXFjn0Fiv7xaWctKJ+M0fI/aHoDdGFnAAABFVZhcmlhbnRJbmZvX0lubGluZVJlZhVWYXJpYW50
SW5mb19JbmxpbmVSZWYFAgRuYW1lAQlGaWVsZE5hbWW8Vt2BDia7b5GB9tJQNcWOfQWK/vFpZy0on4zR
8j9oegJ0eQEJSW5saW5lUmVmrYUZO5cj88a0HcOWeaAtmr8Vlo0JOMF8UegcLz5mAJsWVmFyaWFudElu
Zm9fSW5saW5lUmVmMRZWYXJpYW50SW5mb19JbmxpbmVSZWYxBQIEbmFtZQEJRmllbGROYW1lvFbdgQ4m
u2+RgfbSUDXFjn0Fiv7xaWctKJ+M0fI/aHoCdHkBCklubGluZVJlZjHAlYdDQHMw8YWw4vrm3MBYWRS2
6fc8OamQSoezeMxcgxZWYXJpYW50SW5mb19JbmxpbmVSZWYyFlZhcmlhbnRJbmZvX0lubGluZVJlZjIF
AgRuYW1lAQlGaWVsZE5hbWW8Vt2BDia7b5GB9tJQNcWOfQWK/vFpZy0on4zR8j9oegJ0eQEKSW5saW5l
UmVmMstGRfr5fhYzR1QtE2JX0FPF/ZvTnzdRoNIpAbwrVWREEVZhcmlhbnRJbmZvX0tleVR5EVZhcmlh
bnRJbmZvX0tleVR5BQIEbmFtZQEJRmllbGROYW1lvFbdgQ4mu2+RgfbSUDXFjn0Fiv7xaWctKJ+M0fI/
aHoCdHkBBUtleVR5IaKK9yIUj6NHHOdiONrIDa6XmlGnVxbW2ksii4HuHZMSVmFyaWFudEluZm9fTGli
UmVmElZhcmlhbnRJbmZvX0xpYlJlZgUCBG5hbWUBCUZpZWxkTmFtZbxW3YEOJrtvkYH20lA1xY59BYr+
8WlnLSifjNHyP2h6AnR5AQZMaWJSZWYS02nKMxpzHCdJtUZOyTyD37bDSj85Gf4lzal1mXYRfQ==

----- END STRICT TYPE LIB -----
```