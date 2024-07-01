---
title: Python Trick
abbrlink: 9d5f329
date: 2024-06-26 09:00:00
---

## deepcopy 性能提升

### 1. 循环赋值构造新的对象代替 deepcopy
速度也可以。

### 2. 使用 pickle
```python
data = pickle.loads(pickle.dumps(data))
```

### 3. 使用 json 或者 ujson
```
data = json.loads(json.dumps(my_dict))
```

### 4. 使用 cpickle
```
import _pickle as cPickle
utxos = cPickle.loads(cPickle.dumps(cache_utxos))
```

## pytest 相关
```
pytest ./tests/lib -s
```
