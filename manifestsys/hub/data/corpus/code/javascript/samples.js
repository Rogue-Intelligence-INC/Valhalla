// JavaScript 代码样本 - 用于 Valhalla 训练

// Promise 和 async/await
async function fetchWithRetry(url, retries = 3) {
    for (let i = 0; i < retries; i++) {
        try {
            const response = await fetch(url);
            if (!response.ok) throw new Error(`HTTP ${response.status}`);
            return await response.json();
        } catch (error) {
            if (i === retries - 1) throw error;
            await new Promise(r => setTimeout(r, 1000 * (i + 1)));
        }
    }
}

// 类和继承
class EventEmitter {
    constructor() {
        this.events = new Map();
    }
    
    on(event, callback) {
        if (!this.events.has(event)) {
            this.events.set(event, []);
        }
        this.events.get(event).push(callback);
        return this;
    }
    
    emit(event, ...args) {
        if (this.events.has(event)) {
            this.events.get(event).forEach(cb => cb(...args));
        }
    }
}

// 函数式编程
const compose = (...fns) => x => fns.reduceRight((acc, fn) => fn(acc), x);
const pipe = (...fns) => x => fns.reduce((acc, fn) => fn(acc), x);

const double = x => x * 2;
const increment = x => x + 1;
const square = x => x * x;

// 解构和展开
const mergeObjects = (obj1, obj2) => ({
    ...obj1,
    ...obj2,
    merged: true,
    timestamp: Date.now()
});

// 闭包
function createCounter(initial = 0) {
    let count = initial;
    return {
        increment: () => ++count,
        decrement: () => --count,
        get: () => count,
        reset: () => count = initial
    };
}

// 代理和反射
const createReactiveObject = (target) => {
    return new Proxy(target, {
        get(obj, prop) {
            console.log(`Getting ${prop}`);
            return Reflect.get(obj, prop);
        },
        set(obj, prop, value) {
            console.log(`Setting ${prop} to ${value}`);
            return Reflect.set(obj, prop, value);
        }
    });
};
