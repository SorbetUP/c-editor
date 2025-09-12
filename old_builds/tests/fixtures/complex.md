# Complex Document Test

This document tests edge cases and complex combinations.

## Nested Formatting Edge Cases

- **Bold with *italic* inside**: This should work properly
- *Italic with **bold** inside*: This should also work
- ==Highlight with **bold** and *italic* inside==
- ++Underline with ==highlight== inside++

## Problematic Cases (Previously Failed)

These cases used to create raw markers:

- `==**==` → Should show empty text with highlight
- `++**++` → Should show empty text with underline  
- `***bold**` → Should split correctly
- `**italic*` → Should handle unmatched markers

## Mixed Table

| **Header 1** | *Header 2* | ==Header 3== |
|-------------|------------|--------------|
| Normal      | **Bold**   | *Italic*     |
| ==High==    | ++Under++  | ***Both***   |
| ![img](url) | Text       | Mixed **b** *i* |

## Complex Image Cases

![Complex Image](https://example.com/test.png){w=400 h=300 a=0.7 align=center}

Text with ![inline](icon.png){w=20 h=20} image and **bold ![another](small.png){w=10 h=10} image** in bold.

## Unicode + Formatting

**Unicode bold**: 🔥🌟⭐
*Unicode italic*: café résumé 
==Unicode highlight==: 文字 漢字
++Unicode underline++: العربية עברית

## End

This concludes the complex test document with various edge cases.