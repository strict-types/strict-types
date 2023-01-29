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
`stl:ETZBi44SufHxHZ4A3BqhyuSc8NuJbDTs7zssxPPZSY9x#minus-germany-concert`

```Haskell
namespace StEn -- minus-germany-concert-ETZBi44SufHxHZ4A3BqhyuSc8NuJbDTs7zssxPPZSY9x.stl

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
data KeyTy            :: primitive (Primitive) | enum (EnumVariants) | array (U16) | unicode (Sizing) | ascii (Sizing) | bytes (Sizing)
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
Id: ETZBi44SufHxHZ4A3BqhyuSc8NuJbDTs7zssxPPZSY9x
Checksum: minus-germany-concert

BFN0RW4AMwAADUJ1aWxkRnJhZ21lbnQNQnVpbGRGcmFnbWVudAMCAAVpZGVudAAFAQEFSWRlbnQJ0jba
5bRz0h/g092c7l8PJ/wk2gUH6EbB5xsCjUlVhAEGZGlnaXRzAAUBAAAQCkRlcGVuZGVuY3kKRGVwZW5k
ZW5jeQQDAmlkAQlUeXBlTGliSWSf5IiaTKQbDE8EEhzGNFzFbT7ypr0miyB7qE49WKRIawRuYW1lAQdM
aWJOYW1l4yS5cnw20BhcDsI7K5gKKFbV/tAfnHrfy8jgivoXqsEDdmVyAQZTZW1WZXJGHq42WWTvTPhJ
gKkRA/DUQB7t8gr7SGr6c7Ero3i8mQxFbnVtVmFyaWFudHMMRW51bVZhcmlhbnRzBQEACAEHVmFyaWFu
dDgURrZK0zzyYiWZGwaxevMfwbjPYHRNM7S+NGA7SUM5AQD/AAlGaWVsZE5hbWUJRmllbGROYW1lBQEB
BUlkZW50CdI22uW0c9If4NPdnO5fDyf8JNoFB+hGwecbAo1JVYQPRmllbGRfSW5saW5lUmVmD0ZpZWxk
X0lubGluZVJlZgQCBG5hbWUBCUZpZWxkTmFtZbxW3YEOJrtvkYH20lA1xY59BYr+8WlnLSifjNHyP2h6
AnR5AQlJbmxpbmVSZWYDtsX90nrLvSvZRz9zDxFb7+0U3YuOpexz5ushaiuxVBBGaWVsZF9JbmxpbmVS
ZWYxEEZpZWxkX0lubGluZVJlZjEEAgRuYW1lAQlGaWVsZE5hbWW8Vt2BDia7b5GB9tJQNcWOfQWK/vFp
Zy0on4zR8j9oegJ0eQEKSW5saW5lUmVmMS7lEsvAl7za0QTgllW+XmqsFx9q8jljv7z0vLn6A++YEEZp
ZWxkX0lubGluZVJlZjIQRmllbGRfSW5saW5lUmVmMgQCBG5hbWUBCUZpZWxkTmFtZbxW3YEOJrtvkYH2
0lA1xY59BYr+8WlnLSifjNHyP2h6AnR5AQpJbmxpbmVSZWYyOFzJ4qlkAkTvEnpZDlZqll2wzj/75+uk
ES5sNVLB2AsLRmllbGRfS2V5VHkLRmllbGRfS2V5VHkEAgRuYW1lAQlGaWVsZE5hbWW8Vt2BDia7b5GB
9tJQNcWOfQWK/vFpZy0on4zR8j9oegJ0eQEFS2V5VHkInKTAyzA6WXkRxQX1lvVztSKs089V//bb/6Km
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
jD3zWnbVSBVOYW1lZEZpZWxkc19JbmxpbmVSZWYVTmFtZWRGaWVsZHNfSW5saW5lUmVmBQEABwEPRmll
bGRfSW5saW5lUmVmt43Z0H2Zu3p2UQxtDgyacyjIiDHtmbRKEdAGV9/SAuUBAP8AFk5hbWVkRmllbGRz
X0lubGluZVJlZjEWTmFtZWRGaWVsZHNfSW5saW5lUmVmMQUBAAcBEEZpZWxkX0lubGluZVJlZjGMbN6F
tT7amTzfjnIV0+MUVZSv43+SslyLFnDYtwZHKwEA/wAWTmFtZWRGaWVsZHNfSW5saW5lUmVmMhZOYW1l
ZEZpZWxkc19JbmxpbmVSZWYyBQEABwEQRmllbGRfSW5saW5lUmVmMhkItiJzoXF9j1f8OEdNa1UO6l5u
jsVirEULyUyTYzAiAQD/ABFOYW1lZEZpZWxkc19LZXlUeRFOYW1lZEZpZWxkc19LZXlUeQUBAAcBC0Zp
ZWxkX0tleVR5KL1lt+K9+dAcYQsxIqCdzPQMapXjvt8LU2PTFwZMt/QBAP8AEk5hbWVkRmllbGRzX0xp
YlJlZhJOYW1lZEZpZWxkc19MaWJSZWYFAQAHAQxGaWVsZF9MaWJSZWZEpFUJ58fWkZQDwkSOk5ltuvrZ
HHglIg7u4UBveYjl+gEA/wALUHJlRnJhZ21lbnQLUHJlRnJhZ21lbnQDAgAFaWRlbnQABQEBBUlkZW50
CdI22uW0c9If4NPdnO5fDyf8JNoFB+hGwecbAo1JVYQBBmRpZ2l0cwAFAQAAEAlQcmltaXRpdmUJUHJp
bWl0aXZlBQEAAAEFU2VtSWQFU2VtSWQFAQAGAAABIAAGU2VtVmVyBlNlbVZlcgQFBW1pbm9yAAACBW1h
am9yAAACBXBhdGNoAAACA3ByZQAHAQtQcmVGcmFnbWVudEGvontcfrlK1YpARXW2vPABNknLg2NbOJgu
SeYou7qgAAD/AAVidWlsZAAHAQ1CdWlsZEZyYWdtZW508/LRtIONeVBHQC4ZMp6TD2rAWULk5LyAHk2Y
Kp4adIQAAP8ABlNpemluZwZTaXppbmcEAgNtaW4AAAIDbWF4AAACDFR5X0lubGluZVJlZgxUeV9Jbmxp
bmVSZWYDCgAJcHJpbWl0aXZlAAUBAQlQcmltaXRpdmUrxVyrp1/UV7gbwapXRgFljhZe4GFjR0heVesP
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
/IO6jAAA/wAFdHlwZXMACQABAQdMaWJUeXBlsAlTw4Fv+tBjbNEXfiSfG7bm6VkbztCds1RFsM4EOuUB
AP//CVR5cGVMaWJJZAlUeXBlTGliSWQFAQAGAAABIAAIVHlwZU5hbWUIVHlwZU5hbWUFAQEFSWRlbnQJ
0jba5bRz0h/g092c7l8PJ/wk2gUH6EbB5xsCjUlVhBdVbmlvblZhcmlhbnRzX0lubGluZVJlZhdVbmlv
blZhcmlhbnRzX0lubGluZVJlZgUBAAkAAQEVVmFyaWFudEluZm9fSW5saW5lUmVm45gHub/LWy72wvya
+/qvxlKNABo538HnQTcQfArIPfkAAP8AGFVuaW9uVmFyaWFudHNfSW5saW5lUmVmMRhVbmlvblZhcmlh
bnRzX0lubGluZVJlZjEFAQAJAAEBFlZhcmlhbnRJbmZvX0lubGluZVJlZjFiDLWRkLtpXEPEE5R9fl16
fu5J4FmJPm7yrk54T4X0dAAA/wAYVW5pb25WYXJpYW50c19JbmxpbmVSZWYyGFVuaW9uVmFyaWFudHNf
SW5saW5lUmVmMgUBAAkAAQEWVmFyaWFudEluZm9fSW5saW5lUmVmMh54tCybl7or6ONURFMabFM9L3N6
Ybx8fw8yAdbTMzZeAAD/ABNVbmlvblZhcmlhbnRzX0tleVR5E1VuaW9uVmFyaWFudHNfS2V5VHkFAQAJ
AAEBEVZhcmlhbnRJbmZvX0tleVR5h3SbaW+wQ9biKFm+f/re1hJkug3bLYlNxKLCkfpD424AAP8AFFVu
aW9uVmFyaWFudHNfTGliUmVmFFVuaW9uVmFyaWFudHNfTGliUmVmBQEACQABARJWYXJpYW50SW5mb19M
aWJSZWY230MwO+j7MCqRk9QPy2W859nAy6GbUBnPv3nbVyXEkgAA/wAXVW5uYW1lZEZpZWxkc19Jbmxp
bmVSZWYXVW5uYW1lZEZpZWxkc19JbmxpbmVSZWYFAQAHAQlJbmxpbmVSZWYDtsX90nrLvSvZRz9zDxFb
7+0U3YuOpexz5ushaiuxVAEA/wAYVW5uYW1lZEZpZWxkc19JbmxpbmVSZWYxGFVubmFtZWRGaWVsZHNf
SW5saW5lUmVmMQUBAAcBCklubGluZVJlZjEu5RLLwJe82tEE4JZVvl5qrBcfavI5Y7+89Ly5+gPvmAEA
/wAYVW5uYW1lZEZpZWxkc19JbmxpbmVSZWYyGFVubmFtZWRGaWVsZHNfSW5saW5lUmVmMgUBAAcBCklu
bGluZVJlZjI4XMniqWQCRO8SelkOVmqWXbDOP/vn66QRLmw1UsHYCwEA/wATVW5uYW1lZEZpZWxkc19L
ZXlUeRNVbm5hbWVkRmllbGRzX0tleVR5BQEABwEFS2V5VHkInKTAyzA6WXkRxQX1lvVztSKs089V//bb
/6Kmr36AWQEA/wAUVW5uYW1lZEZpZWxkc19MaWJSZWYUVW5uYW1lZEZpZWxkc19MaWJSZWYFAQAHAQZM
aWJSZWYDa93kqmxePg0wLhOhdpaUwMpk3fMvqdLCFkrEwAgsqgEA/wAHVmFyaWFudAdWYXJpYW50BAIE
bmFtZQEJRmllbGROYW1lvFbdgQ4mu2+RgfbSUDXFjn0Fiv7xaWctKJ+M0fI/aHoDb3JkAAABFVZhcmlh
bnRJbmZvX0lubGluZVJlZhVWYXJpYW50SW5mb19JbmxpbmVSZWYEAgRuYW1lAQlGaWVsZE5hbWW8Vt2B
Dia7b5GB9tJQNcWOfQWK/vFpZy0on4zR8j9oegJ0eQEJSW5saW5lUmVmA7bF/dJ6y70r2Uc/cw8RW+/t
FN2LjqXsc+brIWorsVQWVmFyaWFudEluZm9fSW5saW5lUmVmMRZWYXJpYW50SW5mb19JbmxpbmVSZWYx
BAIEbmFtZQEJRmllbGROYW1lvFbdgQ4mu2+RgfbSUDXFjn0Fiv7xaWctKJ+M0fI/aHoCdHkBCklubGlu
ZVJlZjEu5RLLwJe82tEE4JZVvl5qrBcfavI5Y7+89Ly5+gPvmBZWYXJpYW50SW5mb19JbmxpbmVSZWYy
FlZhcmlhbnRJbmZvX0lubGluZVJlZjIEAgRuYW1lAQlGaWVsZE5hbWW8Vt2BDia7b5GB9tJQNcWOfQWK
/vFpZy0on4zR8j9oegJ0eQEKSW5saW5lUmVmMjhcyeKpZAJE7xJ6WQ5WapZdsM4/++frpBEubDVSwdgL
EVZhcmlhbnRJbmZvX0tleVR5EVZhcmlhbnRJbmZvX0tleVR5BAIEbmFtZQEJRmllbGROYW1lvFbdgQ4m
u2+RgfbSUDXFjn0Fiv7xaWctKJ+M0fI/aHoCdHkBBUtleVR5CJykwMswOll5EcUF9Zb1c7UirNPPVf/2
2/+ipq9+gFkSVmFyaWFudEluZm9fTGliUmVmElZhcmlhbnRJbmZvX0xpYlJlZgQCBG5hbWUBCUZpZWxk
TmFtZbxW3YEOJrtvkYH20lA1xY59BYr+8WlnLSifjNHyP2h6AnR5AQZMaWJSZWYDa93kqmxePg0wLhOh
dpaUwMpk3fMvqdLCFkrEwAgsqg==

----- END STRICT TYPE LIB -----
```