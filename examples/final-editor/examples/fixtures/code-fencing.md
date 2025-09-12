# Code Fencing Test Document

## JavaScript Code

```javascript
function fibonacci(n) {
  if (n <= 1) return n;
  return fibonacci(n - 1) + fibonacci(n - 2);
}

console.log(fibonacci(10));
```

## Python Code

```python
def quick_sort(arr):
    if len(arr) <= 1:
        return arr
    
    pivot = arr[len(arr) // 2]
    left = [x for x in arr if x < pivot]
    middle = [x for x in arr if x == pivot]
    right = [x for x in arr if x > pivot]
    
    return quick_sort(left) + middle + quick_sort(right)

print(quick_sort([3, 6, 8, 10, 1, 2, 1]))
```

## C Code

```c
#include <stdio.h>
#include <stdlib.h>

int main() {
    int *arr = malloc(5 * sizeof(int));
    for (int i = 0; i < 5; i++) {
        arr[i] = i * i;
        printf("%d ", arr[i]);
    }
    free(arr);
    return 0;
}
```

## Plain Code Block

```
No syntax highlighting here
Just plain monospace text
  With some indentation
    And more indentation
```

## Inline Code

Here we have some `inline code` mixed with regular text and more `code snippets`.