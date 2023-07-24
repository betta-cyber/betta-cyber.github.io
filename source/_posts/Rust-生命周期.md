---
title: Rust 生命周期
abbrlink: '20368356'
date: 2022-03-24 09:00:00
---

学习生命周期参数的意义是，避免出现悬垂指针。

在全球70%的安全漏洞里面，悬垂指针可能占50%，所以避免出现悬垂指针是一个很重要的安全保障。

### 悬垂指针的成因

在许多编程语言中（比如C），显示地从内存中删除一个对象或者返回时通过销毁栈帧，并不会改变相关的指针的值。该指针仍旧指向内存中相同的位置，即使引用已经被删除，现在可能已经挪作他用。

```
{
   char *dp = NULL;
   /* ... */
   {
       char c;
       dp = &c;
   } /* c falls out of scope */
   /* dp is now a dangling pointer */
}
```

如果操作系统能够侦测运行时的指向空指针的引用，一个方案是在内部快消失之前给dp赋为0（NULL）。另一个方案是保证dp在没有被初始化之前不再被使用。

另一个常见原因是混用 malloc() 和 free()：当一个指针指向的内存被释放后就会变成悬垂指针。正如上个例子，可以避免这个问题的一种方法是在释放它的引用后把指针重置为NULL。

```
#include <stdlib.h>
void func()
{
    char *dp = malloc(A_CONST);
    /* ... */
    free(dp);         /* dp now becomes a dangling pointer */
    dp = NULL;        /* dp is no longer dangling */
    /* ... */
}
```

一个很常见的失误是返回一个栈分配的局部变量：一旦调用的函数返回了，分配给这些变量的空间被回收，此时它们拥有的是“垃圾值”。

```
int *func(void)
{
    int num = 1234;
    /* ... */
    return &num;
}
```

调用 func 后，尝试从该指针暂时能读取到正确的值（1234），但是再次调用函数后将会重写栈为 num 分配的的值，再从该指针读取的值就不正确了。如果必须要返回一个指向 num 的指针，num 的作用域必须大于这个函数——它也许被声明为 static。


### 什么是生命周期

```
fn main() {
  let a;                // -------------+-- a start
  {                     //              |
    let b = 5;          // -+-- b start |
    a = &b;             //  |           |
  }                     // -+-- b over  |
  println!("a: {}", a); //              |
}                       // -------------+-- a over
```

上面第 5 行代码把变量b 借给了变量a，所以 a 是借用方，b 是出借方。可以发现变量a（借用方）的生命周期比变量b（出借方）的生命周期长，于是这样做违背了 rust 的借用规则（借用方的生命周期不能比出借方的生命周期还要长）。因为当 b 在生命周期结束时，a 还是保持了对 b 的借用，就会导致 a 所指向的那块内存空间已经被释放了，那么变量 a 就会是一个悬垂引用。

对于一个参数和返回值都包含引用的函数而言，该函数的参数是出借方，函数返回值所绑定到的那个变量就是借用方。所以这种函数也需要满足借用规则（借用方的生命周期不能比出借方的生命周期还要长）。那么就需要对函数返回值的生命周期进行标注，告知编译器函数返回值的生命周期信息。

```
fn max_num(x: &i32, y: &i32) -> &i32 {
  if x > y {
    &x
  } else {
    &y
  }
}

fn main() {
  let x = 1;                // -------------+-- x start
  let max;                  // -------------+-- max start
  {                         //              |
    let y = 8;              // -------------+-- y start
    max = max_num(&x, &y);  //              |
  }                         // -------------+-- y over
  println!("max: {}", max); //              |
}                           // -------------+-- max, x over
```

由于缺少生命周期参数，编译器不知道 max_num 函数返回的引用生命周期是什么，所以运行报错：

函数的生命周期参数声明在函数名后的尖括号 <> 里，然后每个参数名跟在一个单引号' 后面，多个参数用逗号隔开。如果在参数和返回值的地方需要使用生命周期进行标注时，只需要在 & 符号后面加上一个单引号' 和之前声明的参数名即可。生命周期参数名可以是任意合法的名称。例如：

```
fn max_num<'a>(x: &'a i32, y: &'a i32) -> &'a i32 {
  if x > y {
    &x
  } else {
    &y
  }
}
fn main() {
  let x = 1;                // -------------+-- x start
  let max;                  // -------------+-- max start
  {                         //              |
    let y = 8;              // -------------+-- y start
    max = max_num(&x, &y);  //              |
  }                         // -------------+-- y over
  println!("max: {}", max); //              |
}                           // -------------+-- max, x over
```

上面代码对函数 max_num 的参数和返回值的生命周期进行了标注，用于告诉编译器函数参数和函数返回值的生命周期一样长。在第 13 行代码对 max_num 进行调用时，编译器会把变量 x 的生命周期和变量 y 的生命周期与 max_num 函数的生命周期参数'a 建立关联。这里值得注意的是，变量 x 和变量 y 的生命周期长短其实是不一样的，那么关联到 max_num 函数的生命周期参数'a 的长度是多少呢？实际上编译器会取变量 x 的生命周期和变量 y 的生命周期重叠的部分，也就是取最短的那个变量的生命周期与'a 建立关联。这里最短的生命周期是变量 y，所以'a 关联的生命周期就是变量 y 的生命周期。

运行上面代码，会有报错信息：

```
error[E0597]: `y` does not live long enough
  --> src/main.rs:13:27
   |
13 |         max = max_num(&x, &y);
   |                           ^^ borrowed value does not live long enough
14 |     }
   |     - `y` dropped here while still borrowed
15 |     println!("max: {}", max);
   |                         --- borrow later used here
```

报错信息说变量 y 的生命周期不够长，当 y 的生命周期结束后，仍然被借用。

我们仔细观察发现 max_num 函数返回值所绑定到的那个变量 max（借用方）的生命周期是从第 10 行代码到第 16 行代码，而 max_num 函数的返回值（出借方）的生命周期是'a，'a 的生命周期又是变量 x 的生命周期和变量 y 的生命周期中最短的那个，也就是变量 y 的生命周期。变量 y 的生命周期是代码的第 12 行到第 14 行。所以这里不满足借用规则（借用方的生命周期不能比出借方的生命周期还要长）。也就是为什么编译器会说变量 y 的生命周期不够长的原因了。函数的生命周期参数并不会改变生命周期的长短，只是用于编译来判断是否满足借用规则。

将代码做如下调整，使其变量 max 的生命周期小于变量 y 的生命周期，编译器就可以正常通过：


```
fn max_num<'a>(x: &'a i32, y: &'a i32) -> &'a i32 {
  if x > y {
    &x
  } else {
    &y
  }
}
fn main() {
  let x = 1;                  // -------------+-- x start
  let y = 8;                  // -------------+-- y start
  let max = max_num(&x, &y);  // -------------+-- max start
  println!("max: {}", max);   //              |
}                             // -------------+-- max, y, x over
```

函数存在多个生命周期参数时，需要标注各个参数之间的关系。例如：

```
fn max_num<'a, 'b: 'a>(x: &'a i32, y: &'b i32) -> &'a i32 {
  if x > y {
    &x
  } else {
    &y
  }
}
fn main() {
  let x = 1;                  // -------------+-- x start
  let y = 8;                  // -------------+-- y start
  let max = max_num(&x, &y);  // -------------+-- max start
  println!("max: {}", max);   //              |
}                             // -------------+-- max, y, x over
```

上面代码使用'b: 'a 来标注'a 与'b 之间的生命周期关系，它表示'a 的生命周期不能超过'b，即函数返回值的生命周期'a（借用方）不能超过'b``（出借方），‘a 也不会超过‘a`（出借方）。

### 结构体中的生命周期参数

一个包含引用成员的结构体，必须保证结构体本身的生命周期不能超过任何一个引用成员的生命周期。否则就会出现成员已经被销毁之后，结构体还保持对那个成员的引用就会产生悬垂引用。所以这依旧是 rust 的借用规则即借用方（结构体本身）的生命周期不能比出借方（结构体中的引用成员）的生命周期还要长。因此就需要在声明结构体的同时也声明生命周期参数，同时对结构体的引用成员进行生命周期参数标注。

结构体生命周期参数声明在结构体名称后的尖括号 <> 里，每个参数名跟在一个单引号' 后面，多个参数用逗号隔开。在进行标注时，只需要在引用成员的 & 符号后面加上一个单引号' 和之前声明的参数名即可。生命周期参数名可以是任意合法的名称。例如：

```
struct Foo<'a> {
    v: &'a i32
}'>
```

### 静态生命周期参数

有一个特殊的生命周期参数叫 static，它的生命周期是整个应用程序。跟其他生命周期参数不同的是，它是表示一个具体的生命周期长度，而不是泛指。static 生命周期的变量存储在静态段中。

所有的字符串字面值都是 'static 生命周期，例如：'

```
let s: &'static str = "s is a static lifetime.";'
```

上面代码中的生命周期参数可以省略，就变成如下形式：

```
let s: &str = "s is a static lifetime.";
```

### 与编译器做斗争


假设我们有一个整数数组，我们想遍历偶数。我们可以使用Iterator::filter()方法，让我们尝试手动实现它，因为这样做将使我们对Rust的生命周期规则有更深入的了解。

代码如下：
```
struct Numbers<'a> {
    data: &'a Vec<i32>,
    even_idx: usize,
}

impl<'a> Numbers<'a> {
    pub fn new(data: &'a Vec<i32>) -> Self {
        Self{ data, even_idx: 0 }
    }

    pub fn next_even(&mut self) -> Option<&i32> {
        while let Some(x) = self.get(self.even_idx) {
            self.even_idx += 1;
            if *x % 2 == 0 { return Some(x); }
        }
        None
    }

    fn get(&self, idx: usize) -> Option<&i32> {
        if idx < self.data.len() { 
            Some(&self.data[idx])
        } else {
            None
        }
    }
}

fn main() {
    let xs = vec![1,2,3,4,5,6,7,8,9];
    let mut numbers = Numbers::new(&xs);
    while let Some(x) = numbers.next_even() {
        println!("{}", x);
    }
}
```
首先，注意struct Number<'a>的生命周期说明符'a。这是必需的，因为Number结构体有一个对vector的引用，即data: &'a Vec<i32>。换句话说，如果原始数据Vector超出作用域，结构体Number就不能存在。这在new(data: &'a Vec<i32>)方法签名中也很明显。

这里的生命周期'a并不表示Number对象本身的生命周期。它是原始Vector实例的生命周期！

让我们看看编译器对上面的代码是怎么说的：

```
error[E0506]: cannot assign to `self.even_idx` because it is borrowed
  --> src/main.rs:13:13
   |
11 |     pub fn next_even(&mut self) -> Option<&i32> {
   |                      - let's call the lifetime of this reference `'1`
12 |         while let Some(x) = self.get(self.even_idx) {
   |                             ----------------------- `self.even_idx` is borrowed here
13 |             self.even_idx += 1;
   |             ^^^^^^^^^^^^^^^^^^ `self.even_idx` is assigned to here but it was already borrowed
14 |             if *x % 2 == 0 { return Some(x); }
   |                                     ------- returning this value requires that `*self` is borrowed for `'1`
```
让我们尝试从编译器错误消息中理解每个语句。首先，编译器告诉我们应该假设&mut self的生命周期为'1 ，这是Number对象实例本身。如前所述，Number实例生命周期不是'a，这就是编译器给它'1的原因。实际上，用生命周期的名称会让代码更清晰一些，我们将使用与main()函数中的变量名相同的生命周期名称。

```
let xs = vec![1,2,3,4,5,6,7,8,9];
let mut numbers = Numbers::new(&xs);
```

也就是说，对于xs对象，生命周期名称为'xs '，对于numbers对象，生命周期名称为'numbers，这将真正帮助我们理解编译器消息。

代码修改如下：
```
struct Numbers<'xs> {
    data: &'xs Vec<i32>,
    even_idx: usize,
}

impl<'xs> Numbers<'xs> {
    pub fn new(data: &'xs Vec<i32>) -> Self {
        Self{ data, even_idx: 0 }
    }

    pub fn next_even<'numbers>(&'numbers mut self) -> Option<&i32> {
        while let Some(x) = self.get(self.even_idx) {
            self.even_idx += 1;
            if *x % 2 == 0 { return Some(x); }
        }
        None
    }

    fn get<'numbers>(&'numbers self, idx: usize) -> Option<&i32> {
        if idx < self.data.len() { 
            Some(&self.data[idx])
        } else {
            None
        }
    }
}

fn main() {
    let xs = vec![1,2,3,4,5,6,7,8,9];
    let mut numbers = Numbers::new(&xs);
    while let Some(x) = numbers.next_even() {
        println!("{}", x);
    }
}
```

现在，让我们再次查看编译器消息。

```
error[E0506]: cannot assign to `self.even_idx` because it is borrowed
  --> src/main.rs:40:13
   |
38 |     pub fn next_even<'numbers>(&'numbers mut self) -> Option<&i32> {
   |                      -------- lifetime `'numbers` defined here
39 |         while let Some(x) = self.get(self.even_idx) {
   |                             ----------------------- `self.even_idx` is borrowed here
40 |             self.even_idx += 1;
   |             ^^^^^^^^^^^^^^^^^^ `self.even_idx` is assigned to here but it was already borrowed
41 |             if *x % 2 == 0 { return Some(x); }
   |                                     ------- returning this value requires that `*self` is borrowed for `'numbers`
```

现在，信息："'self.even_idx' is borrowed here"是有意义的，因为我们的Numbers::get()方法确实借用了Numbers对象本身。由于我们data的生命周期为'xs，因此我们期望返回值的生命周期为'xs，而不是'numbers。



不知何故，Some(x)具有'numbers而不是'xs的生命周期。为什么会这样？跟踪x的来源，我们看到它来自我们的方法Numbers::get()。这是否意味着该方法返回Option<&'numbers i32>而不是Option<&'xs i32>？让我们显式地指定方法next_even()和get()返回的生命周期：

```
pub fn next_even<'numbers>(&'numbers mut self) -> Option<&'xs i32> {
    while let Some(x) = self.get(self.even_idx) {
        self.even_idx += 1;
        if *x % 2 == 0 { return Some(x); }
    }
    None
}

fn get<'numbers>(&'numbers self, idx: usize) -> Option<&'xs i32> {
    if idx < self.data.len() { 
        Some(&self.data[idx])
    } else {
        None
    }
}
```

惊喜！有了这个最后的更改，编译就成功了，我们得到了预期的结果。那么，根本原因是什么？这是因为省略生命周期导致的，基本上，如果Rust可以推断出合理的生命周期，那么生命周期规范可以被省略。不幸的是，这并不总是有效的。在我们的get()方法中，只有一个输入参数&self有生命周期，所以它的输出Option<&i32>被假定为与输入具有相同的生命周期，即'numbers而不是'xs，这就是问题的根源。

**当然，直接去掉 numbers 的生命周期写法也是完全可以运行的。省略的话，应该是默认的生命周期**


总结:
1，结构体的生命周期参数与它的实例存在多长时间无关。
2，省略生命周期有时会引入歧义或意外错误。

