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
`stl:A3bA2b3orwYexYywJGdUJ1qhZX1owzyfbqh2VgNmwkaQ#texas-grand-camel`

```Haskell
namespace StEn -- texas-grand-camel-A3bA2b3orwYexYywJGdUJ1qhZX1owzyfbqh2VgNmwkaQ.stl

-- no dependencies

data BuildFragment    :: (U128)?
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
data KeyTy            :: primitive (Primitive) | unicode (Sizing) | ascii (Sizing) | enum (EnumVariants) | array (U16) | bytes (Sizing)
data LibName          :: (Ident)
data LibRef           :: inline (Ty_InlineRef) | named (TypeName, SemId) | extern (TypeName, LibName, SemId)
data LibType          :: name TypeName, ty Ty_LibRef
data NamedFields_InlineRef :: ([Field_InlineRef ^ 1..0xff])
data NamedFields_InlineRef1 :: ([Field_InlineRef1 ^ 1..0xff])
data NamedFields_InlineRef2 :: ([Field_InlineRef2 ^ 1..0xff])
data NamedFields_KeyTy :: ([Field_KeyTy ^ 1..0xff])
data NamedFields_LibRef :: ([Field_LibRef ^ 1..0xff])
data PreFragment      :: (U128)?
data Primitive        :: (U8)
data SemId            :: ([U8 ^ 32])
data SemVer           :: minor U16, major U16, patch U16, pre [PreFragment ^ ..255], build [BuildFragment ^ ..255]
data Sizing           :: min U16, max U16
data Ty_InlineRef     :: primitive (Primitive) | unicode () | enum (EnumVariants) | union (UnionVariants_InlineRef) | struct (NamedFields_InlineRef) | tuple (UnnamedFields_InlineRef) | array (InlineRef, U16) | list (InlineRef, Sizing) | set (InlineRef, Sizing) | map (KeyTy, InlineRef, Sizing)
data Ty_InlineRef1    :: primitive (Primitive) | unicode () | enum (EnumVariants) | union (UnionVariants_InlineRef1) | struct (NamedFields_InlineRef1) | tuple (UnnamedFields_InlineRef1) | array (InlineRef1, U16) | list (InlineRef1, Sizing) | set (InlineRef1, Sizing) | map (KeyTy, InlineRef1, Sizing)
data Ty_InlineRef2    :: primitive (Primitive) | unicode () | enum (EnumVariants) | union (UnionVariants_InlineRef2) | struct (NamedFields_InlineRef2) | tuple (UnnamedFields_InlineRef2) | array (InlineRef2, U16) | list (InlineRef2, Sizing) | set (InlineRef2, Sizing) | map (KeyTy, InlineRef2, Sizing)
data Ty_KeyTy         :: primitive (Primitive) | unicode () | enum (EnumVariants) | union (UnionVariants_KeyTy) | struct (NamedFields_KeyTy) | tuple (UnnamedFields_KeyTy) | array (KeyTy, U16) | list (KeyTy, Sizing) | set (KeyTy, Sizing) | map (KeyTy, KeyTy, Sizing)
data Ty_LibRef        :: primitive (Primitive) | unicode () | enum (EnumVariants) | union (UnionVariants_LibRef) | struct (NamedFields_LibRef) | tuple (UnnamedFields_LibRef) | array (LibRef, U16) | list (LibRef, Sizing) | set (LibRef, Sizing) | map (KeyTy, LibRef, Sizing)
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
data Variant          :: name FieldName, ord U8
data VariantInfo_InlineRef :: name FieldName, ty InlineRef
data VariantInfo_InlineRef1 :: name FieldName, ty InlineRef1
data VariantInfo_InlineRef2 :: name FieldName, ty InlineRef2
data VariantInfo_KeyTy :: name FieldName, ty KeyTy
data VariantInfo_LibRef :: name FieldName, ty LibRef
```

Encoded library:
```
----- BEGIN STRICT TYPE LIB -----
Id: A3bA2b3orwYexYywJGdUJ1qhZX1owzyfbqh2VgNmwkaQ
Checksum: texas-grand-camel

BFN0RW4AMwAADUJ1aWxkRnJhZ21lbnQNQnVpbGRGcmFnbWVudAQCAAVpZGVudAAGAQEFSWRlbnQJ0jba
5bRz0h/g092c7l8PJ/wk2gUH6EbB5xsCjUlVhAEGZGlnaXRzAAYBAAAQCkRlcGVuZGVuY3kKRGVwZW5k
ZW5jeQUDAmlkAQlUeXBlTGliSWSf5IiaTKQbDE8EEhzGNFzFbT7ypr0miyB7qE49WKRIawRuYW1lAQdM
aWJOYW1l4yS5cnw20BhcDsI7K5gKKFbV/tAfnHrfy8jgivoXqsEDdmVyAQZTZW1WZXJGHq42WWTvTPhJ
gKkRA/DUQB7t8gr7SGr6c7Ero3i8mQxFbnVtVmFyaWFudHMMRW51bVZhcmlhbnRzBgEACQEHVmFyaWFu
dDgURrZK0zzyYiWZGwaxevMfwbjPYHRNM7S+NGA7SUM5AQD/AAlGaWVsZE5hbWUJRmllbGROYW1lBgEB
BUlkZW50CdI22uW0c9If4NPdnO5fDyf8JNoFB+hGwecbAo1JVYQPRmllbGRfSW5saW5lUmVmD0ZpZWxk
X0lubGluZVJlZgUCBG5hbWUBCUZpZWxkTmFtZbxW3YEOJrtvkYH20lA1xY59BYr+8WlnLSifjNHyP2h6
AnR5AQlJbmxpbmVSZWY4joYgYued8o6FTay4GyRlNzfvkdCWYOtGOCK8KJja7BBGaWVsZF9JbmxpbmVS
ZWYxEEZpZWxkX0lubGluZVJlZjEFAgRuYW1lAQlGaWVsZE5hbWW8Vt2BDia7b5GB9tJQNcWOfQWK/vFp
Zy0on4zR8j9oegJ0eQEKSW5saW5lUmVmMSLmAJASrxTJuOxMBo+SttiairMD0NzoRXCEul6To2DtEEZp
ZWxkX0lubGluZVJlZjIQRmllbGRfSW5saW5lUmVmMgUCBG5hbWUBCUZpZWxkTmFtZbxW3YEOJrtvkYH2
0lA1xY59BYr+8WlnLSifjNHyP2h6AnR5AQpJbmxpbmVSZWYyAMzH/v1gf5SXyuVG0+2ehBF/xNjLlc3s
Y7OCO1RxMuoLRmllbGRfS2V5VHkLRmllbGRfS2V5VHkFAgRuYW1lAQlGaWVsZE5hbWW8Vt2BDia7b5GB
9tJQNcWOfQWK/vFpZy0on4zR8j9oegJ0eQEFS2V5VHlDdLls076OMM5IEOn+bPwjS+4yrLfPhxSdNqT+
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
hA02A/g2yRVOYW1lZEZpZWxkc19JbmxpbmVSZWYVTmFtZWRGaWVsZHNfSW5saW5lUmVmBgEACAEPRmll
bGRfSW5saW5lUmVmXOfZME1P5oztltFClNDqyhq0VYWTmoTCT/espNnKkvQBAP8AFk5hbWVkRmllbGRz
X0lubGluZVJlZjEWTmFtZWRGaWVsZHNfSW5saW5lUmVmMQYBAAgBEEZpZWxkX0lubGluZVJlZjEzQeE2
3EHKLJ/r/vMetFN1br3CUsnKZDbowu3cNIqEmgEA/wAWTmFtZWRGaWVsZHNfSW5saW5lUmVmMhZOYW1l
ZEZpZWxkc19JbmxpbmVSZWYyBgEACAEQRmllbGRfSW5saW5lUmVmMv9l2zqbnTj1Q6/CGQ8Rb3a5oOIg
19bulu9ASXyeZ6WaAQD/ABFOYW1lZEZpZWxkc19LZXlUeRFOYW1lZEZpZWxkc19LZXlUeQYBAAgBC0Zp
ZWxkX0tleVR5dpBUK4DUu8kUo9OEa64cLJq/hOIXpqUz227rJ+f0IRwBAP8AEk5hbWVkRmllbGRzX0xp
YlJlZhJOYW1lZEZpZWxkc19MaWJSZWYGAQAIAQxGaWVsZF9MaWJSZWbDbyJi/25PBQunRy5vKSQZPAuR
5XbSGiNOLgXxJ+GwXgEA/wALUHJlRnJhZ21lbnQLUHJlRnJhZ21lbnQEAgAFaWRlbnQABgEBBUlkZW50
CdI22uW0c9If4NPdnO5fDyf8JNoFB+hGwecbAo1JVYQBBmRpZ2l0cwAGAQAAEAlQcmltaXRpdmUJUHJp
bWl0aXZlBgEAAAEFU2VtSWQFU2VtSWQGAQAHAAABIAAGU2VtVmVyBlNlbVZlcgUFBW1pbm9yAAACBW1h
am9yAAACBXBhdGNoAAACA3ByZQAIAQtQcmVGcmFnbWVudEGvontcfrlK1YpARXW2vPABNknLg2NbOJgu
SeYou7qgAAD/AAVidWlsZAAIAQ1CdWlsZEZyYWdtZW508/LRtIONeVBHQC4ZMp6TD2rAWULk5LyAHk2Y
Kp4adIQAAP8ABlNpemluZwZTaXppbmcFAgNtaW4AAAIDbWF4AAACDFR5X0lubGluZVJlZgxUeV9Jbmxp
bmVSZWYECgAJcHJpbWl0aXZlAAYBAQlQcmltaXRpdmUrxVyrp1/UV7gbwapXRgFljhZe4GFjR0heVesP
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
/IO6jAAA/wAFdHlwZXMACgABAQdMaWJUeXBleFomvjKxpC6gr/w675xuD9zU/wLzRmk8DgVgudWZK1YB
AP//CVR5cGVMaWJJZAlUeXBlTGliSWQGAQAHAAABIAAIVHlwZU5hbWUIVHlwZU5hbWUGAQEFSWRlbnQJ
0jba5bRz0h/g092c7l8PJ/wk2gUH6EbB5xsCjUlVhBdVbmlvblZhcmlhbnRzX0lubGluZVJlZhdVbmlv
blZhcmlhbnRzX0lubGluZVJlZgYBAAoAAQEVVmFyaWFudEluZm9fSW5saW5lUmVmTFMvczdUagDSA36W
PoywS5qRbFiyBH8ZHMFmb4JWjZUAAP8AGFVuaW9uVmFyaWFudHNfSW5saW5lUmVmMRhVbmlvblZhcmlh
bnRzX0lubGluZVJlZjEGAQAKAAEBFlZhcmlhbnRJbmZvX0lubGluZVJlZjErN7M1zod0jngGhQ6MDR1T
JqZ9oVq8QPvtBRRZDK4mJQAA/wAYVW5pb25WYXJpYW50c19JbmxpbmVSZWYyGFVuaW9uVmFyaWFudHNf
SW5saW5lUmVmMgYBAAoAAQEWVmFyaWFudEluZm9fSW5saW5lUmVmMvBGSYMvMOS2G7Q1wJNUrHblvntv
c7Lfsp3HKDK/qLYRAAD/ABNVbmlvblZhcmlhbnRzX0tleVR5E1VuaW9uVmFyaWFudHNfS2V5VHkGAQAK
AAEBEVZhcmlhbnRJbmZvX0tleVR5cl2PP9w30Kb15xfb2wBuEJtnsHbQSkqimT3xVRhj5tQAAP8AFFVu
aW9uVmFyaWFudHNfTGliUmVmFFVuaW9uVmFyaWFudHNfTGliUmVmBgEACgABARJWYXJpYW50SW5mb19M
aWJSZWZg2Zig/qFrgBHy/zp9ys9g0zFxSAu7OlZMt91VolaPSwAA/wAXVW5uYW1lZEZpZWxkc19Jbmxp
bmVSZWYXVW5uYW1lZEZpZWxkc19JbmxpbmVSZWYGAQAIAQlJbmxpbmVSZWY4joYgYued8o6FTay4GyRl
NzfvkdCWYOtGOCK8KJja7AEA/wAYVW5uYW1lZEZpZWxkc19JbmxpbmVSZWYxGFVubmFtZWRGaWVsZHNf
SW5saW5lUmVmMQYBAAgBCklubGluZVJlZjEi5gCQEq8UybjsTAaPkrbYmoqzA9Dc6EVwhLpek6Ng7QEA
/wAYVW5uYW1lZEZpZWxkc19JbmxpbmVSZWYyGFVubmFtZWRGaWVsZHNfSW5saW5lUmVmMgYBAAgBCklu
bGluZVJlZjIAzMf+/WB/lJfK5UbT7Z6EEX/E2MuVzexjs4I7VHEy6gEA/wATVW5uYW1lZEZpZWxkc19L
ZXlUeRNVbm5hbWVkRmllbGRzX0tleVR5BgEACAEFS2V5VHlDdLls076OMM5IEOn+bPwjS+4yrLfPhxSd
NqT+KScLrAEA/wAUVW5uYW1lZEZpZWxkc19MaWJSZWYUVW5uYW1lZEZpZWxkc19MaWJSZWYGAQAIAQZM
aWJSZWa24r3boMvtuG5vB77+f1K1vvy9zQ9IMzpRxYPmv3kdFgEA/wAHVmFyaWFudAdWYXJpYW50BQIE
bmFtZQEJRmllbGROYW1lvFbdgQ4mu2+RgfbSUDXFjn0Fiv7xaWctKJ+M0fI/aHoDb3JkAAABFVZhcmlh
bnRJbmZvX0lubGluZVJlZhVWYXJpYW50SW5mb19JbmxpbmVSZWYFAgRuYW1lAQlGaWVsZE5hbWW8Vt2B
Dia7b5GB9tJQNcWOfQWK/vFpZy0on4zR8j9oegJ0eQEJSW5saW5lUmVmOI6GIGLnnfKOhU2suBskZTc3
75HQlmDrRjgivCiY2uwWVmFyaWFudEluZm9fSW5saW5lUmVmMRZWYXJpYW50SW5mb19JbmxpbmVSZWYx
BQIEbmFtZQEJRmllbGROYW1lvFbdgQ4mu2+RgfbSUDXFjn0Fiv7xaWctKJ+M0fI/aHoCdHkBCklubGlu
ZVJlZjEi5gCQEq8UybjsTAaPkrbYmoqzA9Dc6EVwhLpek6Ng7RZWYXJpYW50SW5mb19JbmxpbmVSZWYy
FlZhcmlhbnRJbmZvX0lubGluZVJlZjIFAgRuYW1lAQlGaWVsZE5hbWW8Vt2BDia7b5GB9tJQNcWOfQWK
/vFpZy0on4zR8j9oegJ0eQEKSW5saW5lUmVmMgDMx/79YH+Ul8rlRtPtnoQRf8TYy5XN7GOzgjtUcTLq
EVZhcmlhbnRJbmZvX0tleVR5EVZhcmlhbnRJbmZvX0tleVR5BQIEbmFtZQEJRmllbGROYW1lvFbdgQ4m
u2+RgfbSUDXFjn0Fiv7xaWctKJ+M0fI/aHoCdHkBBUtleVR5Q3S5bNO+jjDOSBDp/mz8I0vuMqy3z4cU
nTak/iknC6wSVmFyaWFudEluZm9fTGliUmVmElZhcmlhbnRJbmZvX0xpYlJlZgUCBG5hbWUBCUZpZWxk
TmFtZbxW3YEOJrtvkYH20lA1xY59BYr+8WlnLSifjNHyP2h6AnR5AQZMaWJSZWa24r3boMvtuG5vB77+
f1K1vvy9zQ9IMzpRxYPmv3kdFg==

----- END STRICT TYPE LIB -----
```