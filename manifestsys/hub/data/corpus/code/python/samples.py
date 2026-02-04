# Python 代码样本 - 用于 Valhalla 训练

from typing import List, Dict, Optional, Generator
import asyncio
from dataclasses import dataclass
from functools import lru_cache

# 装饰器示例
def timer(func):
    """计时装饰器"""
    import time
    def wrapper(*args, **kwargs):
        start = time.time()
        result = func(*args, **kwargs)
        end = time.time()
        print(f"{func.__name__} 耗时: {end - start:.4f}s")
        return result
    return wrapper

# 数据类
@dataclass
class Point:
    x: float
    y: float
    
    def distance_to(self, other: 'Point') -> float:
        return ((self.x - other.x)**2 + (self.y - other.y)**2) ** 0.5

# 生成器
def fibonacci_generator() -> Generator[int, None, None]:
    a, b = 0, 1
    while True:
        yield a
        a, b = b, a + b

# 缓存
@lru_cache(maxsize=128)
def factorial(n: int) -> int:
    if n <= 1:
        return 1
    return n * factorial(n - 1)

# 异步编程
async def fetch_multiple(urls: List[str]) -> List[str]:
    async def fetch_one(url: str) -> str:
        await asyncio.sleep(0.1)  # 模拟网络延迟
        return f"Response from {url}"
    
    tasks = [fetch_one(url) for url in urls]
    return await asyncio.gather(*tasks)

# 类继承
class Animal:
    def __init__(self, name: str):
        self.name = name
    
    def speak(self) -> str:
        raise NotImplementedError

class Dog(Animal):
    def speak(self) -> str:
        return f"{self.name} says: Woof!"

class Cat(Animal):
    def speak(self) -> str:
        return f"{self.name} says: Meow!"

# 上下文管理器
class FileHandler:
    def __init__(self, filename: str, mode: str):
        self.filename = filename
        self.mode = mode
        self.file = None
    
    def __enter__(self):
        self.file = open(self.filename, self.mode)
        return self.file
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        if self.file:
            self.file.close()

# 机器学习示例
def gradient_descent(X, y, learning_rate=0.01, iterations=1000):
    """简单线性回归梯度下降"""
    import numpy as np
    m = len(y)
    theta = np.zeros(X.shape[1])
    
    for _ in range(iterations):
        predictions = X.dot(theta)
        errors = predictions - y
        gradient = (1/m) * X.T.dot(errors)
        theta -= learning_rate * gradient
    
    return theta
