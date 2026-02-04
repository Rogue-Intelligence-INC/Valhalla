// 🌌 Hello Cosmic-C 示例
// 展示 .vh 文件的基本语法和自适应特性

use std.io

// 自适应容器演示
@adaptive
let messages = Container<string>()

// 高性能计算演示
@optimize
fn fibonacci(n: int) -> int {
    if n <= 1 {
        return n
    }
    return fibonacci(n - 1) + fibonacci(n - 2)
}

fn main() {
    println("🌌 Hello, Cosmic Universe!")
    println("这是 .vh 文件格式的演示")
    
    // 演示自适应容器
    for i in 0..5 {
        messages.add("Hello " + i)
    }
    
    // 演示计算优化
    let result = fibonacci(10)
    println("Fibonacci(10) = " + result)
    
    println("✨ Fate-Voyager 已自动优化程序性能!")
} 