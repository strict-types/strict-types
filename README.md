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
`stl:9PAgDBAAAGt41sxDmkmXksGHYbVuz4N2zcFiyPnVqQbv#mama-jumbo-sinatra`

Import this lib by putting in the file header
`typelib 9PAgDBAAAGt41sxDmkmXksGHYbVuz4N2zcFiyPnVqQbv#mama-jumbo-sinatra`

Source code (you can save it to 
`mama-jumbo-sinatra.9PAgDBAAAGt41sxDmkmXksGHYbVuz4N2zcFiyPnVqQbv.sty` file)

```Haskell
{-
  Id: 9PAgDBAAAGt41sxDmkmXksGHYbVuz4N2zcFiyPnVqQbv
  Name: StrictTypes
  Description: Library implementing strict types abstract syntax tree
  Author: Dr Maxim Orlovsky <orlovsky@ubideco.org>
  Copyright: (C) UBIDECO Institute, 2022-2023, Switzerland. All rights reserved.
  License: Apache-2.0
  Checksum: `mama-jumbo-sinatra
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
`mama-jumbo-sinatra.9PAgDBAAAGt41sxDmkmXksGHYbVuz4N2zcFiyPnVqQbv.stl`):

```
----- BEGIN STRICT TYPE LIB -----
Id: 9PAgDBAAAGt41sxDmkmXksGHYbVuz4N2zcFiyPnVqQbv
Checksum: mama-jumbo-sinatra

C1N0cmljdFR5cGVzADMAAA1CdWlsZEZyYWdtZW50DUJ1aWxkRnJhZ21lbnQEAgAFaWRlbnQABQEBBUlk
ZW504TuN1tWttWWvFkVuW4Q4cSncj+U/uooL09iZrqs7bbQBBmRpZ2l0cwAFAQEFSWRlbnThO43W1a21
Za8WRW5bhDhxKdyP5T+6igvT2JmuqztttApEZXBlbmRlbmN5CkRlcGVuZGVuY3kGAwJpZAEJVHlwZUxp
Yklk5tZzBPpj+/Vr6BbBThcI4gFCKEoTe/Nrr16VLLPaTbIEbmFtZQEHTGliTmFtZci7OpO8xnS/nrmi
2hQxVsmC+zQa9pS3hmn/jOpQxBtLA3ZlcgEGU2VtVmVykAEc2f/8Fyx1Ercnhl0Ktvczm6VI6bq+Rn9o
XXvzv34MRW51bVZhcmlhbnRzDEVudW1WYXJpYW50cwUBAAkBB1ZhcmlhbnQ30qMQ/YF/EXTEEW/vyZYH
gygtsa+1faA3r3XCJ7PnxQEA/wAJRmllbGROYW1lCUZpZWxkTmFtZQUBAQVJZGVudOE7jdbVrbVlrxZF
bluEOHEp3I/lP7qKC9PYma6rO220D0ZpZWxkX0lubGluZVJlZg9GaWVsZF9JbmxpbmVSZWYGAgRuYW1l
AQlGaWVsZE5hbWWebYfERZ09VnGbDiZJNu7jPRNYs+6bd2AA0qV8i/EPdgJ0eQEJSW5saW5lUmVm0UpC
TwlPGa0Sl1lcJhVfhek4C/P3H35IN7+/sEvvnBQQRmllbGRfSW5saW5lUmVmMRBGaWVsZF9JbmxpbmVS
ZWYxBgIEbmFtZQEJRmllbGROYW1lnm2HxEWdPVZxmw4mSTbu4z0TWLPum3dgANKlfIvxD3YCdHkBCklu
bGluZVJlZjEuYUEnW4tJVG26kP6zETQAB8JOsHhVKCQ9/Zxq5DeQjxBGaWVsZF9JbmxpbmVSZWYyEEZp
ZWxkX0lubGluZVJlZjIGAgRuYW1lAQlGaWVsZE5hbWWebYfERZ09VnGbDiZJNu7jPRNYs+6bd2AA0qV8
i/EPdgJ0eQEKSW5saW5lUmVmMj7/ZEodNtb2h8QYRzshPYKpduxuImJFG4aBohZC+ZXjC0ZpZWxkX0tl
eVR5C0ZpZWxkX0tleVR5BgIEbmFtZQEJRmllbGROYW1lnm2HxEWdPVZxmw4mSTbu4z0TWLPum3dgANKl
fIvxD3YCdHkBBUtleVR5U8sTzjpSj5pt7X8+EVo/xqzHGaFqkGryOQbd+NrkX+UMRmllbGRfTGliUmVm
DEZpZWxkX0xpYlJlZgYCBG5hbWUBCUZpZWxkTmFtZZ5th8RFnT1WcZsOJkk27uM9E1iz7pt3YADSpXyL
8Q92AnR5AQZMaWJSZWZtCth5Kdr+Mbuw++PoWUlY1goI3BQAuMi/MyKx36uitAVJZGVudAVJZGVudAUB
AAABCUlubGluZVJlZglJbmxpbmVSZWYEAwAGaW5saW5lAAUBAQ1UeV9JbmxpbmVSZWYxuZfUYPkGS0al
Trs2bazwvpj2UjJe2XHj75R8n8NWREMBBW5hbWVkAAUCAQhUeXBlTmFtZeCbZ1fMHERKYfyKmU/KahwE
hxRLzmSZTnWQ+/SdbFYCAQVTZW1JZOURd04eQP2C876g43oH2gEZLSWfHfuswMWAF8kTAJztAgZleHRl
cm4ABQMBCFR5cGVOYW1l4JtnV8wcREph/IqZT8pqHASHFEvOZJlOdZD79J1sVgIBB0xpYk5hbWXIuzqT
vMZ0v565otoUMVbJgvs0GvaUt4Zp/4zqUMQbSwEFU2VtSWTlEXdOHkD9gvO+oON6B9oBGS0lnx37rMDF
gBfJEwCc7QpJbmxpbmVSZWYxCklubGluZVJlZjEEAwAGaW5saW5lAAUBAQ1UeV9JbmxpbmVSZWYyl8Mz
ApuZEhK/mQZwMlHPSA5JK+2u9YjhLIzQhrSTQt8BBW5hbWVkAAUCAQhUeXBlTmFtZeCbZ1fMHERKYfyK
mU/KahwEhxRLzmSZTnWQ+/SdbFYCAQVTZW1JZOURd04eQP2C876g43oH2gEZLSWfHfuswMWAF8kTAJzt
AgZleHRlcm4ABQMBCFR5cGVOYW1l4JtnV8wcREph/IqZT8pqHASHFEvOZJlOdZD79J1sVgIBB0xpYk5h
bWXIuzqTvMZ0v565otoUMVbJgvs0GvaUt4Zp/4zqUMQbSwEFU2VtSWTlEXdOHkD9gvO+oON6B9oBGS0l
nx37rMDFgBfJEwCc7QpJbmxpbmVSZWYyCklubGluZVJlZjIEAwAGaW5saW5lAAUBAQhUeV9LZXlUeZWE
vXC92skfH/EWG0UyWnSUhyUN2wSlxh+NjB81UB+RAQVuYW1lZAAFAgEIVHlwZU5hbWXgm2dXzBxESmH8
iplPymocBIcUS85kmU51kPv0nWxWAgEFU2VtSWTlEXdOHkD9gvO+oON6B9oBGS0lnx37rMDFgBfJEwCc
7QIGZXh0ZXJuAAUDAQhUeXBlTmFtZeCbZ1fMHERKYfyKmU/KahwEhxRLzmSZTnWQ+/SdbFYCAQdMaWJO
YW1lyLs6k7zGdL+euaLaFDFWyYL7NBr2lLeGaf+M6lDEG0sBBVNlbUlk5RF3Th5A/YLzvqDjegfaARkt
JZ8d+6zAxYAXyRMAnO0FS2V5VHkFS2V5VHkEBgAJcHJpbWl0aXZlAAUBAQlQcmltaXRpdmV+Uo/CbZst
8mB21M8VvVvVboomLdHv27YbTq4/ZKp75wMEZW51bQAFAQEMRW51bVZhcmlhbnRzDsT4z1ytlIaynvrK
/LAHsAlZ4Pxbq3vxs8cN9O0cjGQHBWFycmF5AAUBAAACEAd1bmljb2RlAAUBAQZTaXppbmfqi5zC7p3l
yQWh9Df9iUN3q7wbzE73ZeHjE32mWCqtGBEFYXNjaWkABQEBBlNpemluZ+qLnMLuneXJBaH0N/2JQ3er
vBvMTvdl4eMTfaZYKq0YEgVieXRlcwAFAQEGU2l6aW5n6oucwu6d5ckFofQ3/YlDd6u8G8xO92Xh4xN9
plgqrRgHTGliTmFtZQdMaWJOYW1lBQEBBUlkZW504TuN1tWttWWvFkVuW4Q4cSncj+U/uooL09iZrqs7
bbQGTGliUmVmBkxpYlJlZgQDAAZpbmxpbmUABQEBDFR5X0lubGluZVJlZipnDyVs0on0LQCVsDlOHxBi
uR25Bk90saKgr3LrcZi5AQVuYW1lZAAFAgEIVHlwZU5hbWXgm2dXzBxESmH8iplPymocBIcUS85kmU51
kPv0nWxWAgEFU2VtSWTlEXdOHkD9gvO+oON6B9oBGS0lnx37rMDFgBfJEwCc7QIGZXh0ZXJuAAUDAQhU
eXBlTmFtZeCbZ1fMHERKYfyKmU/KahwEhxRLzmSZTnWQ+/SdbFYCAQdMaWJOYW1lyLs6k7zGdL+euaLa
FDFWyYL7NBr2lLeGaf+M6lDEG0sBBVNlbUlk5RF3Th5A/YLzvqDjegfaARktJZ8d+6zAxYAXyRMAnO0H
TGliVHlwZQdMaWJUeXBlBgIEbmFtZQEIVHlwZU5hbWXgm2dXzBxESmH8iplPymocBIcUS85kmU51kPv0
nWxWAgJ0eQEJVHlfTGliUmVmLu5CBXMlja0dtoXCkg4LczndTGulzrP0XaY7TWmSFE4VTmFtZWRGaWVs
ZHNfSW5saW5lUmVmFU5hbWVkRmllbGRzX0lubGluZVJlZgUBAAgBD0ZpZWxkX0lubGluZVJlZrUEM/7k
aDDoRz0I9VkTDCQB6L4zPFAnxCsO2n1W5CVZAQD/ABZOYW1lZEZpZWxkc19JbmxpbmVSZWYxFk5hbWVk
RmllbGRzX0lubGluZVJlZjEFAQAIARBGaWVsZF9JbmxpbmVSZWYxm/RYdkh+K8OuVqCN6qZgPEFQvCRh
xjsZrL0K3uPxTZMBAP8AFk5hbWVkRmllbGRzX0lubGluZVJlZjIWTmFtZWRGaWVsZHNfSW5saW5lUmVm
MgUBAAgBEEZpZWxkX0lubGluZVJlZjLnnFoBS63lbP5fq6WmziwatzCsE0BZUl1UJ3UntICevQEA/wAR
TmFtZWRGaWVsZHNfS2V5VHkRTmFtZWRGaWVsZHNfS2V5VHkFAQAIAQtGaWVsZF9LZXlUeeRP/RZk/+9A
Khe9c5mJCmQxDRkO8lVehFRSUXTLBPgUAQD/ABJOYW1lZEZpZWxkc19MaWJSZWYSTmFtZWRGaWVsZHNf
TGliUmVmBQEACAEMRmllbGRfTGliUmVmE3dxbZJQdf90SZBbQSzgAgAA0A6jvEk4So7wy+avGygBAP8A
C1ByZUZyYWdtZW50C1ByZUZyYWdtZW50BAIABWlkZW50AAUBAQVJZGVudOE7jdbVrbVlrxZFbluEOHEp
3I/lP7qKC9PYma6rO220AQZkaWdpdHMABQEAABAJUHJpbWl0aXZlCVByaW1pdGl2ZQUBAAABBVNlbUlk
BVNlbUlkBQEABwAAASAABlNlbVZlcgZTZW1WZXIGBQVtYWpvcgAAAgVtaW5vcgAAAgVwYXRjaAAAAgNw
cmUACAELUHJlRnJhZ21lbnTr566ZxhAoxJp3giFarP2D8rR9lPiG0RmY8rCTsWS1EwAA/wAFYnVpbGQA
CAENQnVpbGRGcmFnbWVudBFoFzGn5nq5iBSqYyIUKUzyddvYMs9rJE/3fIxkXppsAAD/AAZTaXppbmcG
U2l6aW5nBgIDbWluAAACA21heAAAAgxUeV9JbmxpbmVSZWYMVHlfSW5saW5lUmVmBAoACXByaW1pdGl2
ZQAFAQEJUHJpbWl0aXZlflKPwm2bLfJgdtTPFb1b1W6KJi3R79u2G06uP2Sqe+cBB3VuaWNvZGUAAAAD
BGVudW0ABQEBDEVudW1WYXJpYW50cw7E+M9crZSGsp76yvywB7AJWeD8W6t78bPHDfTtHIxkBAV1bmlv
bgAFAQEXVW5pb25WYXJpYW50c19JbmxpbmVSZWb3KpGG3Dg5QlbZDvKDCw735PCy7NraKn7rEw986BpF
TQUFdHVwbGUABQEBF1VubmFtZWRGaWVsZHNfSW5saW5lUmVm+VVoSJRf9GFcXhnWfvIsc9FeMTxcPDEf
hY1Zaa7hRA0GBnN0cnVjdAAFAQEVTmFtZWRGaWVsZHNfSW5saW5lUmVm5WTxwN2V9d4SzMCGe2W5FrWz
QYMoSj7uqeDvB8OW6MMHBWFycmF5AAUCAQlJbmxpbmVSZWbRSkJPCU8ZrRKXWVwmFV+F6TgL8/cffkg3
v7+wS++cFAAAAggEbGlzdAAFAgEJSW5saW5lUmVm0UpCTwlPGa0Sl1lcJhVfhek4C/P3H35IN7+/sEvv
nBQBBlNpemluZ+qLnMLuneXJBaH0N/2JQ3ervBvMTvdl4eMTfaZYKq0YCQNzZXQABQIBCUlubGluZVJl
ZtFKQk8JTxmtEpdZXCYVX4XpOAvz9x9+SDe/v7BL75wUAQZTaXppbmfqi5zC7p3lyQWh9Df9iUN3q7wb
zE73ZeHjE32mWCqtGAoDbWFwAAUDAQVLZXlUeVPLE846Uo+abe1/PhFaP8asxxmhapBq8jkG3fja5F/l
AQlJbmxpbmVSZWbRSkJPCU8ZrRKXWVwmFV+F6TgL8/cffkg3v7+wS++cFAEGU2l6aW5n6oucwu6d5ckF
ofQ3/YlDd6u8G8xO92Xh4xN9plgqrRgNVHlfSW5saW5lUmVmMQ1UeV9JbmxpbmVSZWYxBAoACXByaW1p
dGl2ZQAFAQEJUHJpbWl0aXZlflKPwm2bLfJgdtTPFb1b1W6KJi3R79u2G06uP2Sqe+cBB3VuaWNvZGUA
AAADBGVudW0ABQEBDEVudW1WYXJpYW50cw7E+M9crZSGsp76yvywB7AJWeD8W6t78bPHDfTtHIxkBAV1
bmlvbgAFAQEYVW5pb25WYXJpYW50c19JbmxpbmVSZWYx1dg4c3no6JCQDZIJHWQEAkOoY4XAcHMTvINw
0wFMUeUFBXR1cGxlAAUBARhVbm5hbWVkRmllbGRzX0lubGluZVJlZjGmENAN75QKp2Nwoo7ciJ011hs9
yuQhIPWafeYmnywy6AYGc3RydWN0AAUBARZOYW1lZEZpZWxkc19JbmxpbmVSZWYxBtg+27koqvzJM6SO
HKisE4OWVmGXCxZliM5YA0riB+QHBWFycmF5AAUCAQpJbmxpbmVSZWYxLmFBJ1uLSVRtupD+sxE0AAfC
TrB4VSgkPf2cauQ3kI8AAAIIBGxpc3QABQIBCklubGluZVJlZjEuYUEnW4tJVG26kP6zETQAB8JOsHhV
KCQ9/Zxq5DeQjwEGU2l6aW5n6oucwu6d5ckFofQ3/YlDd6u8G8xO92Xh4xN9plgqrRgJA3NldAAFAgEK
SW5saW5lUmVmMS5hQSdbi0lUbbqQ/rMRNAAHwk6weFUoJD39nGrkN5CPAQZTaXppbmfqi5zC7p3lyQWh
9Df9iUN3q7wbzE73ZeHjE32mWCqtGAoDbWFwAAUDAQVLZXlUeVPLE846Uo+abe1/PhFaP8asxxmhapBq
8jkG3fja5F/lAQpJbmxpbmVSZWYxLmFBJ1uLSVRtupD+sxE0AAfCTrB4VSgkPf2cauQ3kI8BBlNpemlu
Z+qLnMLuneXJBaH0N/2JQ3ervBvMTvdl4eMTfaZYKq0YDVR5X0lubGluZVJlZjINVHlfSW5saW5lUmVm
MgQKAAlwcmltaXRpdmUABQEBCVByaW1pdGl2ZX5Sj8Jtmy3yYHbUzxW9W9VuiiYt0e/bthtOrj9kqnvn
AQd1bmljb2RlAAAAAwRlbnVtAAUBAQxFbnVtVmFyaWFudHMOxPjPXK2UhrKe+sr8sAewCVng/Fure/Gz
xw307RyMZAQFdW5pb24ABQEBGFVuaW9uVmFyaWFudHNfSW5saW5lUmVmMu1RMDv1j5DlDXaRFqpzFAAP
iTabT+Fa8jF76sH720HPBQV0dXBsZQAFAQEYVW5uYW1lZEZpZWxkc19JbmxpbmVSZWYyzqIA8TekJu3L
TP6EUuLUFPpkMI0f7cbzwiBg1CN1iGQGBnN0cnVjdAAFAQEWTmFtZWRGaWVsZHNfSW5saW5lUmVmMksj
Rk+CjvWDNnsPoH0f+b5cNrw7hFYHrq64/oSxL4IaBwVhcnJheQAFAgEKSW5saW5lUmVmMj7/ZEodNtb2
h8QYRzshPYKpduxuImJFG4aBohZC+ZXjAAACCARsaXN0AAUCAQpJbmxpbmVSZWYyPv9kSh021vaHxBhH
OyE9gql27G4iYkUbhoGiFkL5leMBBlNpemluZ+qLnMLuneXJBaH0N/2JQ3ervBvMTvdl4eMTfaZYKq0Y
CQNzZXQABQIBCklubGluZVJlZjI+/2RKHTbW9ofEGEc7IT2CqXbsbiJiRRuGgaIWQvmV4wEGU2l6aW5n
6oucwu6d5ckFofQ3/YlDd6u8G8xO92Xh4xN9plgqrRgKA21hcAAFAwEFS2V5VHlTyxPOOlKPmm3tfz4R
Wj/GrMcZoWqQavI5Bt342uRf5QEKSW5saW5lUmVmMj7/ZEodNtb2h8QYRzshPYKpduxuImJFG4aBohZC
+ZXjAQZTaXppbmfqi5zC7p3lyQWh9Df9iUN3q7wbzE73ZeHjE32mWCqtGAhUeV9LZXlUeQhUeV9LZXlU
eQQKAAlwcmltaXRpdmUABQEBCVByaW1pdGl2ZX5Sj8Jtmy3yYHbUzxW9W9VuiiYt0e/bthtOrj9kqnvn
AQd1bmljb2RlAAAAAwRlbnVtAAUBAQxFbnVtVmFyaWFudHMOxPjPXK2UhrKe+sr8sAewCVng/Fure/Gz
xw307RyMZAQFdW5pb24ABQEBE1VuaW9uVmFyaWFudHNfS2V5VHksqIeoFKbSTnF+e2EneQT1euY7D0Ri
T5o2I4WEvs/JhwUFdHVwbGUABQEBE1VubmFtZWRGaWVsZHNfS2V5VHmUIExl+64ZU1oDWEHGqpcye6pQ
G9cvFDf1zj+mqo9C/wYGc3RydWN0AAUBARFOYW1lZEZpZWxkc19LZXlUeTJCFHXseF1+NqBHtJKBT94i
OMyh4mZ+UcgwA9H6jFoqBwVhcnJheQAFAgEFS2V5VHlTyxPOOlKPmm3tfz4RWj/GrMcZoWqQavI5Bt34
2uRf5QAAAggEbGlzdAAFAgEFS2V5VHlTyxPOOlKPmm3tfz4RWj/GrMcZoWqQavI5Bt342uRf5QEGU2l6
aW5n6oucwu6d5ckFofQ3/YlDd6u8G8xO92Xh4xN9plgqrRgJA3NldAAFAgEFS2V5VHlTyxPOOlKPmm3t
fz4RWj/GrMcZoWqQavI5Bt342uRf5QEGU2l6aW5n6oucwu6d5ckFofQ3/YlDd6u8G8xO92Xh4xN9plgq
rRgKA21hcAAFAwEFS2V5VHlTyxPOOlKPmm3tfz4RWj/GrMcZoWqQavI5Bt342uRf5QEFS2V5VHlTyxPO
OlKPmm3tfz4RWj/GrMcZoWqQavI5Bt342uRf5QEGU2l6aW5n6oucwu6d5ckFofQ3/YlDd6u8G8xO92Xh
4xN9plgqrRgJVHlfTGliUmVmCVR5X0xpYlJlZgQKAAlwcmltaXRpdmUABQEBCVByaW1pdGl2ZX5Sj8Jt
my3yYHbUzxW9W9VuiiYt0e/bthtOrj9kqnvnAQd1bmljb2RlAAAAAwRlbnVtAAUBAQxFbnVtVmFyaWFu
dHMOxPjPXK2UhrKe+sr8sAewCVng/Fure/Gzxw307RyMZAQFdW5pb24ABQEBFFVuaW9uVmFyaWFudHNf
TGliUmVmkaL6YjwKErgkvUlytvbrtl399NDtgpeOVfTAUCPWdQcFBXR1cGxlAAUBARRVbm5hbWVkRmll
bGRzX0xpYlJlZnNjhG8tTAmtKNObLsYjVyQujUGtI3VJNUUIOMnNqXFVBgZzdHJ1Y3QABQEBEk5hbWVk
RmllbGRzX0xpYlJlZqXfSrNkxE4CuKyhRn0kZ7xKzDuNZye9yC/3knwoABsKBwVhcnJheQAFAgEGTGli
UmVmbQrYeSna/jG7sPvj6FlJWNYKCNwUALjIvzMisd+rorQAAAIIBGxpc3QABQIBBkxpYlJlZm0K2Hkp
2v4xu7D74+hZSVjWCgjcFAC4yL8zIrHfq6K0AQZTaXppbmfqi5zC7p3lyQWh9Df9iUN3q7wbzE73ZeHj
E32mWCqtGAkDc2V0AAUCAQZMaWJSZWZtCth5Kdr+Mbuw++PoWUlY1goI3BQAuMi/MyKx36uitAEGU2l6
aW5n6oucwu6d5ckFofQ3/YlDd6u8G8xO92Xh4xN9plgqrRgKA21hcAAFAwEFS2V5VHlTyxPOOlKPmm3t
fz4RWj/GrMcZoWqQavI5Bt342uRf5QEGTGliUmVmbQrYeSna/jG7sPvj6FlJWNYKCNwUALjIvzMisd+r
orQBBlNpemluZ+qLnMLuneXJBaH0N/2JQ3ervBvMTvdl4eMTfaZYKq0YB1R5cGVMaWIHVHlwZUxpYgYD
BG5hbWUBB0xpYk5hbWXIuzqTvMZ0v565otoUMVbJgvs0GvaUt4Zp/4zqUMQbSwxkZXBlbmRlbmNpZXMA
CgABAQpEZXBlbmRlbmN50+S4bppZz39p7RJqAxwNUp+kxNpg3VekxLYZyo1JV1UAAP8ABXR5cGVzAAoA
AQEHTGliVHlwZb5g/BDpnv3m1Z61rDvv/rU/1MXE/lJ2vXIQiRHT9N/eAQD//wlUeXBlTGliSWQJVHlw
ZUxpYklkBQEABwAAASAACFR5cGVOYW1lCFR5cGVOYW1lBQEBBUlkZW504TuN1tWttWWvFkVuW4Q4cSnc
j+U/uooL09iZrqs7bbQXVW5pb25WYXJpYW50c19JbmxpbmVSZWYXVW5pb25WYXJpYW50c19JbmxpbmVS
ZWYFAQAKAAEBFVZhcmlhbnRJbmZvX0lubGluZVJlZiYxKM6K/nG/UtvrFlK72FzGjdmzVnp23JzamAS2
d9K5AAD/ABhVbmlvblZhcmlhbnRzX0lubGluZVJlZjEYVW5pb25WYXJpYW50c19JbmxpbmVSZWYxBQEA
CgABARZWYXJpYW50SW5mb19JbmxpbmVSZWYxoLW0GT90S6TmWeAciNYUvOF+dHUFZUjKP0HEHCZMVvAA
AP8AGFVuaW9uVmFyaWFudHNfSW5saW5lUmVmMhhVbmlvblZhcmlhbnRzX0lubGluZVJlZjIFAQAKAAEB
FlZhcmlhbnRJbmZvX0lubGluZVJlZjLGv2a2f7C3EZWjtaaQZW2JpsK3aeDDnkriC9W4vqGOTQAA/wAT
VW5pb25WYXJpYW50c19LZXlUeRNVbmlvblZhcmlhbnRzX0tleVR5BQEACgABARFWYXJpYW50SW5mb19L
ZXlUeQoHF7wwBkccag2XT3DLFWxM1JEwLYJdJm1Vt22kvyHiAAD/ABRVbmlvblZhcmlhbnRzX0xpYlJl
ZhRVbmlvblZhcmlhbnRzX0xpYlJlZgUBAAoAAQESVmFyaWFudEluZm9fTGliUmVmH6XOe+XiO8zo4PMs
ZaMLgq3mD+rBx0QDA2X29nZUNowAAP8AF1VubmFtZWRGaWVsZHNfSW5saW5lUmVmF1VubmFtZWRGaWVs
ZHNfSW5saW5lUmVmBQEACAEJSW5saW5lUmVm0UpCTwlPGa0Sl1lcJhVfhek4C/P3H35IN7+/sEvvnBQB
AP8AGFVubmFtZWRGaWVsZHNfSW5saW5lUmVmMRhVbm5hbWVkRmllbGRzX0lubGluZVJlZjEFAQAIAQpJ
bmxpbmVSZWYxLmFBJ1uLSVRtupD+sxE0AAfCTrB4VSgkPf2cauQ3kI8BAP8AGFVubmFtZWRGaWVsZHNf
SW5saW5lUmVmMhhVbm5hbWVkRmllbGRzX0lubGluZVJlZjIFAQAIAQpJbmxpbmVSZWYyPv9kSh021vaH
xBhHOyE9gql27G4iYkUbhoGiFkL5leMBAP8AE1VubmFtZWRGaWVsZHNfS2V5VHkTVW5uYW1lZEZpZWxk
c19LZXlUeQUBAAgBBUtleVR5U8sTzjpSj5pt7X8+EVo/xqzHGaFqkGryOQbd+NrkX+UBAP8AFFVubmFt
ZWRGaWVsZHNfTGliUmVmFFVubmFtZWRGaWVsZHNfTGliUmVmBQEACAEGTGliUmVmbQrYeSna/jG7sPvj
6FlJWNYKCNwUALjIvzMisd+rorQBAP8AB1ZhcmlhbnQHVmFyaWFudAYCBG5hbWUBCUZpZWxkTmFtZZ5t
h8RFnT1WcZsOJkk27uM9E1iz7pt3YADSpXyL8Q92A3RhZwAAARVWYXJpYW50SW5mb19JbmxpbmVSZWYV
VmFyaWFudEluZm9fSW5saW5lUmVmBgIEbmFtZQEJRmllbGROYW1lnm2HxEWdPVZxmw4mSTbu4z0TWLPu
m3dgANKlfIvxD3YCdHkBCUlubGluZVJlZtFKQk8JTxmtEpdZXCYVX4XpOAvz9x9+SDe/v7BL75wUFlZh
cmlhbnRJbmZvX0lubGluZVJlZjEWVmFyaWFudEluZm9fSW5saW5lUmVmMQYCBG5hbWUBCUZpZWxkTmFt
ZZ5th8RFnT1WcZsOJkk27uM9E1iz7pt3YADSpXyL8Q92AnR5AQpJbmxpbmVSZWYxLmFBJ1uLSVRtupD+
sxE0AAfCTrB4VSgkPf2cauQ3kI8WVmFyaWFudEluZm9fSW5saW5lUmVmMhZWYXJpYW50SW5mb19Jbmxp
bmVSZWYyBgIEbmFtZQEJRmllbGROYW1lnm2HxEWdPVZxmw4mSTbu4z0TWLPum3dgANKlfIvxD3YCdHkB
CklubGluZVJlZjI+/2RKHTbW9ofEGEc7IT2CqXbsbiJiRRuGgaIWQvmV4xFWYXJpYW50SW5mb19LZXlU
eRFWYXJpYW50SW5mb19LZXlUeQYCBG5hbWUBCUZpZWxkTmFtZZ5th8RFnT1WcZsOJkk27uM9E1iz7pt3
YADSpXyL8Q92AnR5AQVLZXlUeVPLE846Uo+abe1/PhFaP8asxxmhapBq8jkG3fja5F/lElZhcmlhbnRJ
bmZvX0xpYlJlZhJWYXJpYW50SW5mb19MaWJSZWYGAgRuYW1lAQlGaWVsZE5hbWWebYfERZ09VnGbDiZJ
Nu7jPRNYs+6bd2AA0qV8i/EPdgJ0eQEGTGliUmVmbQrYeSna/jG7sPvj6FlJWNYKCNwUALjIvzMisd+r
orQ=

----- END STRICT TYPE LIB -----
```

## Contributing

[CONTRIBUTING.md](../CONTRIBUTING.md)

## License

The libraries are distributed on the terms of [Apache 2.0 license](LICENSE).

[strict types]: https://strict-types.org
[gadt]: https://en.wikipedia.org/wiki/Algebraic_data_type
[type theory]: https://en.wikipedia.org/wiki/Type_theory
