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
`stl:Cig1ibSP9KX3nEwzdfBKJPB1Nwesnk3BYbNBGFFTmQMg#source-data-shelf`

```Haskell
{-
-- Import this lib by putting in the file header
-- import source_data_shelf_Cig1ibSP9KX3nEwzdfBKJPB1Nwesnk3BYbNBGFFTmQMg
-}
namespace StEn -- source_data_shelf_Cig1ibSP9KX3nEwzdfBKJPB1Nwesnk3BYbNBGFFTmQMg.stl

-- no dependencies

data BuildFragment    :: ident (Ident) | digits (U128)
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
data SemVer           :: minor U16, major U16, patch U16, pre [PreFragment ^ ..255], build [BuildFragment ^ ..255]
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
Id: Cig1ibSP9KX3nEwzdfBKJPB1Nwesnk3BYbNBGFFTmQMg
Checksum: source-data-shelf

BFN0RW4AMwAADUJ1aWxkRnJhZ21lbnQNQnVpbGRGcmFnbWVudAQCAAVpZGVudAAGAQEFSWRlbnQJ0jba
5bRz0h/g092c7l8PJ/wk2gUH6EbB5xsCjUlVhAEGZGlnaXRzAAYBAAAQCkRlcGVuZGVuY3kKRGVwZW5k
ZW5jeQUDAmlkAQlUeXBlTGliSWSf5IiaTKQbDE8EEhzGNFzFbT7ypr0miyB7qE49WKRIawRuYW1lAQdM
aWJOYW1l4yS5cnw20BhcDsI7K5gKKFbV/tAfnHrfy8jgivoXqsEDdmVyAQZTZW1WZXJGHq42WWTvTPhJ
gKkRA/DUQB7t8gr7SGr6c7Ero3i8mQxFbnVtVmFyaWFudHMMRW51bVZhcmlhbnRzBgEACQEHVmFyaWFu
dB2HMLMQbs+ubq50yNiNELz7bf3rsn9N7fF6cQjF1eRXAQD/AAlGaWVsZE5hbWUJRmllbGROYW1lBgEB
BUlkZW50CdI22uW0c9If4NPdnO5fDyf8JNoFB+hGwecbAo1JVYQPRmllbGRfSW5saW5lUmVmD0ZpZWxk
X0lubGluZVJlZgUCBG5hbWUBCUZpZWxkTmFtZbxW3YEOJrtvkYH20lA1xY59BYr+8WlnLSifjNHyP2h6
AnR5AQlJbmxpbmVSZWathRk7lyPzxrQdw5Z5oC2avxWWjQk4wXxR6BwvPmYAmxBGaWVsZF9JbmxpbmVS
ZWYxEEZpZWxkX0lubGluZVJlZjEFAgRuYW1lAQlGaWVsZE5hbWW8Vt2BDia7b5GB9tJQNcWOfQWK/vFp
Zy0on4zR8j9oegJ0eQEKSW5saW5lUmVmMcCVh0NAczDxhbDi+ubcwFhZFLbp9zw5qZBKh7N4zFyDEEZp
ZWxkX0lubGluZVJlZjIQRmllbGRfSW5saW5lUmVmMgUCBG5hbWUBCUZpZWxkTmFtZbxW3YEOJrtvkYH2
0lA1xY59BYr+8WlnLSifjNHyP2h6AnR5AQpJbmxpbmVSZWYyy0ZF+vl+FjNHVC0TYlfQU8X9m9OfN1Gg
0ikBvCtVZEQLRmllbGRfS2V5VHkLRmllbGRfS2V5VHkFAgRuYW1lAQlGaWVsZE5hbWW8Vt2BDia7b5GB
9tJQNcWOfQWK/vFpZy0on4zR8j9oegJ0eQEFS2V5VHkhoor3IhSPo0cc52I42sgNrpeaUadXFtbaSyKL
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
ODeOKroyTBVOYW1lZEZpZWxkc19JbmxpbmVSZWYVTmFtZWRGaWVsZHNfSW5saW5lUmVmBgEACAEPRmll
bGRfSW5saW5lUmVmmnjwfOQAOSyTf+oF2JkgrDcwiMCKdc7/9gLJ0vaq2aoBAP8AFk5hbWVkRmllbGRz
X0lubGluZVJlZjEWTmFtZWRGaWVsZHNfSW5saW5lUmVmMQYBAAgBEEZpZWxkX0lubGluZVJlZjFXQmsG
aUHgMqhcpZqhD0w2ebc6HBS17OjOBamx8rTi2QEA/wAWTmFtZWRGaWVsZHNfSW5saW5lUmVmMhZOYW1l
ZEZpZWxkc19JbmxpbmVSZWYyBgEACAEQRmllbGRfSW5saW5lUmVmMtzT+z8W2EwEJM6FUMVk/s1VD0fo
bY6iATAfRGqCRxwUAQD/ABFOYW1lZEZpZWxkc19LZXlUeRFOYW1lZEZpZWxkc19LZXlUeQYBAAgBC0Zp
ZWxkX0tleVR54Xm5HjtURHethyFQsbynGD4Tyzk/QUuSzv96ADiQMwkBAP8AEk5hbWVkRmllbGRzX0xp
YlJlZhJOYW1lZEZpZWxkc19MaWJSZWYGAQAIAQxGaWVsZF9MaWJSZWa7GB/7kaLHHI6awkGEyJY4E+Ju
6XJCt2w9vwl0FaPLzgEA/wALUHJlRnJhZ21lbnQLUHJlRnJhZ21lbnQEAgAFaWRlbnQABgEBBUlkZW50
CdI22uW0c9If4NPdnO5fDyf8JNoFB+hGwecbAo1JVYQBBmRpZ2l0cwAGAQAAEAlQcmltaXRpdmUJUHJp
bWl0aXZlBgEAAAEFU2VtSWQFU2VtSWQGAQAHAAABIAAGU2VtVmVyBlNlbVZlcgUFBW1pbm9yAAACBW1h
am9yAAACBXBhdGNoAAACA3ByZQAIAQtQcmVGcmFnbWVudEGvontcfrlK1YpARXW2vPABNknLg2NbOJgu
SeYou7qgAAD/AAVidWlsZAAIAQ1CdWlsZEZyYWdtZW508/LRtIONeVBHQC4ZMp6TD2rAWULk5LyAHk2Y
Kp4adIQAAP8ABlNpemluZwZTaXppbmcFAgNtaW4AAAIDbWF4AAACDFR5X0lubGluZVJlZgxUeV9Jbmxp
bmVSZWYECgAJcHJpbWl0aXZlAAYBAQlQcmltaXRpdmUrxVyrp1/UV7gbwapXRgFljhZe4GFjR0heVesP
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
/IO6jAAA/wAFdHlwZXMACgABAQdMaWJUeXBl2ZBtknU1A33kfp9pn5zuGLIDFV0F+3nv6vHu3urQPm0B
AP//CVR5cGVMaWJJZAlUeXBlTGliSWQGAQAHAAABIAAIVHlwZU5hbWUIVHlwZU5hbWUGAQEFSWRlbnQJ
0jba5bRz0h/g092c7l8PJ/wk2gUH6EbB5xsCjUlVhBdVbmlvblZhcmlhbnRzX0lubGluZVJlZhdVbmlv
blZhcmlhbnRzX0lubGluZVJlZgYBAAoAAQEVVmFyaWFudEluZm9fSW5saW5lUmVmTwSLUDg2kEGyzO+0
U8HcUNjGsQmD/EgEAn4ozWunta0AAP8AGFVuaW9uVmFyaWFudHNfSW5saW5lUmVmMRhVbmlvblZhcmlh
bnRzX0lubGluZVJlZjEGAQAKAAEBFlZhcmlhbnRJbmZvX0lubGluZVJlZjECjlqXV1Uyv5ixEnykQiRp
hAUwFb27ZX9JoNFPS4h/KQAA/wAYVW5pb25WYXJpYW50c19JbmxpbmVSZWYyGFVuaW9uVmFyaWFudHNf
SW5saW5lUmVmMgYBAAoAAQEWVmFyaWFudEluZm9fSW5saW5lUmVmMgEVmcS098MiLQGyN3KNiDKPPhXW
Hbarc18mDlM02L8NAAD/ABNVbmlvblZhcmlhbnRzX0tleVR5E1VuaW9uVmFyaWFudHNfS2V5VHkGAQAK
AAEBEVZhcmlhbnRJbmZvX0tleVR5KW2YUr54GIHgzQGz6Li98L6/sDPluqbb6YpatK9VRzgAAP8AFFVu
aW9uVmFyaWFudHNfTGliUmVmFFVuaW9uVmFyaWFudHNfTGliUmVmBgEACgABARJWYXJpYW50SW5mb19M
aWJSZWb4PnLahaojh/PDnslenAtKoHxmcscR25OOC1cO7a3cJwAA/wAXVW5uYW1lZEZpZWxkc19Jbmxp
bmVSZWYXVW5uYW1lZEZpZWxkc19JbmxpbmVSZWYGAQAIAQlJbmxpbmVSZWathRk7lyPzxrQdw5Z5oC2a
vxWWjQk4wXxR6BwvPmYAmwEA/wAYVW5uYW1lZEZpZWxkc19JbmxpbmVSZWYxGFVubmFtZWRGaWVsZHNf
SW5saW5lUmVmMQYBAAgBCklubGluZVJlZjHAlYdDQHMw8YWw4vrm3MBYWRS26fc8OamQSoezeMxcgwEA
/wAYVW5uYW1lZEZpZWxkc19JbmxpbmVSZWYyGFVubmFtZWRGaWVsZHNfSW5saW5lUmVmMgYBAAgBCklu
bGluZVJlZjLLRkX6+X4WM0dULRNiV9BTxf2b0583UaDSKQG8K1VkRAEA/wATVW5uYW1lZEZpZWxkc19L
ZXlUeRNVbm5hbWVkRmllbGRzX0tleVR5BgEACAEFS2V5VHkhoor3IhSPo0cc52I42sgNrpeaUadXFtba
SyKLge4dkwEA/wAUVW5uYW1lZEZpZWxkc19MaWJSZWYUVW5uYW1lZEZpZWxkc19MaWJSZWYGAQAIAQZM
aWJSZWYS02nKMxpzHCdJtUZOyTyD37bDSj85Gf4lzal1mXYRfQEA/wAHVmFyaWFudAdWYXJpYW50BQIE
bmFtZQEJRmllbGROYW1lvFbdgQ4mu2+RgfbSUDXFjn0Fiv7xaWctKJ+M0fI/aHoDdGFnAAABFVZhcmlh
bnRJbmZvX0lubGluZVJlZhVWYXJpYW50SW5mb19JbmxpbmVSZWYFAgRuYW1lAQlGaWVsZE5hbWW8Vt2B
Dia7b5GB9tJQNcWOfQWK/vFpZy0on4zR8j9oegJ0eQEJSW5saW5lUmVmrYUZO5cj88a0HcOWeaAtmr8V
lo0JOMF8UegcLz5mAJsWVmFyaWFudEluZm9fSW5saW5lUmVmMRZWYXJpYW50SW5mb19JbmxpbmVSZWYx
BQIEbmFtZQEJRmllbGROYW1lvFbdgQ4mu2+RgfbSUDXFjn0Fiv7xaWctKJ+M0fI/aHoCdHkBCklubGlu
ZVJlZjHAlYdDQHMw8YWw4vrm3MBYWRS26fc8OamQSoezeMxcgxZWYXJpYW50SW5mb19JbmxpbmVSZWYy
FlZhcmlhbnRJbmZvX0lubGluZVJlZjIFAgRuYW1lAQlGaWVsZE5hbWW8Vt2BDia7b5GB9tJQNcWOfQWK
/vFpZy0on4zR8j9oegJ0eQEKSW5saW5lUmVmMstGRfr5fhYzR1QtE2JX0FPF/ZvTnzdRoNIpAbwrVWRE
EVZhcmlhbnRJbmZvX0tleVR5EVZhcmlhbnRJbmZvX0tleVR5BQIEbmFtZQEJRmllbGROYW1lvFbdgQ4m
u2+RgfbSUDXFjn0Fiv7xaWctKJ+M0fI/aHoCdHkBBUtleVR5IaKK9yIUj6NHHOdiONrIDa6XmlGnVxbW
2ksii4HuHZMSVmFyaWFudEluZm9fTGliUmVmElZhcmlhbnRJbmZvX0xpYlJlZgUCBG5hbWUBCUZpZWxk
TmFtZbxW3YEOJrtvkYH20lA1xY59BYr+8WlnLSifjNHyP2h6AnR5AQZMaWJSZWYS02nKMxpzHCdJtUZO
yTyD37bDSj85Gf4lzal1mXYRfQ==

----- END STRICT TYPE LIB -----
```