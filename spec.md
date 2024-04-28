# FANG Spec Sheet

### Datatypes (done!)
- Integer `int`
- Floating Point `float`
- String `str`
- Boolean `bool`
---

### Variable declaration (no explicit types yet)
```
let a = 1;
let b: int = 2;
let c: float = 3.14159;
let d: str = "fang";
let e: bool = false;
```

### Function declaration (no return types / visibility yet)
```
pub fn a(p: int): int {
	p ** 2
} 
```

### Lambda declaration (not done)
```
let a = |p: int|: int { p ** 2 }; 
```

### Array declaration (not done)
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

### Object declaration (not done)
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