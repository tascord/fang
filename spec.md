# FANG Spec Sheet

### Datatypes
- Integer `int`
- Floating Point `float`
- String `str`
- Boolean `bool`
---

### Variable declaration
```
let a = 1;
let b: int = 2;
let c: float = 3.14159;
let d: str = "fang";
let e: bool = false;
```

### Function declaration
```
pub fn a(p: int): int {
	p ** 2
} 
```

### Lambda declaration
```
let a = |p: int|: int { p ** 2 }; 
```

### Array declaration
```
let a: Arr<str> = [
	"Hello",
	"World",
];

// Heterogenus array typing
let b: Arr<str + int> = [
	123,
	"too boney"
];
```

### Object declaration
```
struct A = {
	author: str,
	views: int,
	stars: float
};

let instance = A {
	author: "Flora",
	views: 12,
	stars: 4.25,
};
```